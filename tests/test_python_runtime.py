"""Exercise actual local environments and fail-closed runtime handover."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'tools/python_runtime.py'


class LocalPythonTests(unittest.TestCase):
    def invoke(self, root, *extra, env=None):
        return subprocess.run([sys.executable, str(SCRIPT), '--root', str(root), *extra],
                              text=True, capture_output=True, env=env)

    def test_real_environment_child_and_reuse_preserve_existing_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            # A fresh CI checkout has no uv-generated inner ignore file.
            # The repository must ignore the entire environment itself.
            (root / '.gitignore').write_bytes((ROOT / '.gitignore').read_bytes())
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            subprocess.run(['git', '-C', str(root), 'add', '.gitignore'], check=True)
            subprocess.run(['git', '-C', str(root), '-c', 'user.name=Runtime Test', '-c', 'user.email=runtime@example.invalid', 'commit', '-qm', 'fixture'], check=True)
            result = self.invoke(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(subprocess.check_output(['git', '-C', str(root), 'status', '--porcelain'], text=True), '')
            receipt = json.loads(result.stdout)
            self.assertEqual(receipt['version'], '3.15.1')
            self.assertEqual(Path(receipt['prefix']), root / '.venv')
            executable = receipt['executable']
            child = subprocess.check_output([executable, '-c', 'import sys; print(sys.version_info[:3]); print(sys.prefix)'], text=True)
            self.assertIn('(3, 15, 1)', child)
            self.assertIn(str(root / '.venv'), child)
            marker = root / '.venv' / 'preserve-me'
            marker.write_text('in-flight recovery state')
            config = (root / '.venv/pyvenv.cfg').read_bytes()
            path_output = root / 'github-path'
            env_output = root / 'github-env'
            second = self.invoke(root, '--github', env=dict(os.environ, GITHUB_PATH=str(path_output), GITHUB_ENV=str(env_output)))
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(path_output.read_text().strip(), str(Path(executable).parent))
            self.assertIn('CAREOPS_PYTHON=' + executable, env_output.read_text())
            self.assertEqual(marker.read_text(), 'in-flight recovery state')
            self.assertEqual((root / '.venv/pyvenv.cfg').read_bytes(), config)

    @unittest.skipIf(os.name == 'nt', 'POSIX bootstrap symlinks are not a Windows capability')
    def test_bootstrap_through_external_symlink_retains_the_standard_library(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            shim = root / 'bin/python3.15'
            shim.parent.mkdir()
            shim.symlink_to(Path(sys._base_executable).resolve())
            project = root / 'project'
            project.mkdir()
            result = subprocess.run([str(shim), str(SCRIPT), '--root', str(project)],
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads(result.stdout)
            self.assertEqual(receipt['version'], '3.15.1')
            self.assertEqual(Path(receipt['prefix']), project / '.venv')
            child = subprocess.run([receipt['executable'], '-I', '-c',
                                    'import encodings, json, pathlib, sys; print(sys.prefix)'],
                                   text=True, capture_output=True)
            self.assertEqual(child.returncode, 0, child.stderr)
            self.assertEqual(child.stdout.strip(), str(project / '.venv'))

    def test_existing_unmanaged_environment_is_not_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / '.venv'
            target.mkdir()
            marker = target / 'preserve-me'
            marker.write_text('private state')
            result = self.invoke(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(marker.read_text(), 'private state')
            self.assertFalse((target / 'pyvenv.cfg').exists())

    def test_base_interpreter_alias_cannot_pass_as_a_local_environment(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / '.venv/bin/python'
            executable.parent.mkdir(parents=True)
            executable.symlink_to(sys._base_executable)
            result = self.invoke(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('prefix', result.stderr)
            self.assertTrue(executable.is_symlink())

    def test_symlink_environment_rejected_without_mutating_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            external = root / 'external'
            external.mkdir()
            (root / '.venv').symlink_to(external, target_is_directory=True)
            result = self.invoke(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(list(external.iterdir()), [])

    def test_wrong_bootstrap_rejects_before_environment_creation(self):
        import importlib.util
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('python_runtime', SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(module.sys, 'version_info', (3, 16, 0, 'alpha', 1)):
                with self.assertRaisesRegex(RuntimeError, '3.15.1'):
                    module.prepare(root)
            self.assertFalse((root / '.venv').exists())

    def test_failed_verification_does_not_export_github_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / '.venv').mkdir()
            output = root / 'path-output'
            env = dict(os.environ, GITHUB_PATH=str(output), GITHUB_ENV=str(root / 'env-output'))
            result = self.invoke(root, '--github', env=env)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(output.exists())

    def test_python_jobs_bootstrap_after_source_and_use_verified_runtime(self):
        import re
        for workflow in sorted((ROOT / '.github/workflows').glob('*.yml')):
            for job in re.split(r'(?m)(?=^  [A-Za-z0-9_-]+:\n)', workflow.read_text()):
                if 'actions/setup-python@' not in job:
                    continue
                with self.subTest(workflow=workflow.name, job=job.splitlines()[0]):
                    self.assertIn('python-version:', job)
                    self.assertIn('3.15.1', job)
                    self.assertLess(job.index('actions/checkout@'), job.index('actions/setup-python@'))
                    guard = 'run: python tools/python_runtime.py --github'
                    self.assertLess(job.index('actions/setup-python@'), job.index(guard))
                    self.assertIn('"$CAREOPS_PYTHON"', job)
                    self.assertNotRegex(job.replace(guard, ''), r'(?m)^\s+(?:run: )?python3?\s')


if __name__ == '__main__':
    unittest.main()
