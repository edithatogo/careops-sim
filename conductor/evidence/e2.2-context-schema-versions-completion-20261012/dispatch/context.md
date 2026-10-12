# Caller-owned context codec schema versions

Status: root-reviewed owner contract, implementation and runtime acceptance pending. Independent GPT-6.1-sol audit and separate ABM worker readback confirm this prerequisite. Accepted DES direct wire source de8afe50322cd708e868543758231359b0926cd3; durable parent acceptance af5b23ced99420b4309a7aee632bf961edff86525eadf84f63ae2969536c5264 at parent e84770f4f3d29f7e35b143658012af8ed94d7b40. ABM candidate 38ac7cb4ea53c657f89444c934c6bb0eb66123e8 is unqualified: original packet retained two failed executions. This separate prerequisite preserves the original ABM packet no-DES-write boundary and required schema2; do not rewrite its historical authority. Bind this contract commit before source dispatch.

## Minimal boundary

Own exactly crates/kairo-ecs-des/src/flow/checkpoint.rs plus new crates/kairo-ecs-des/tests/flow_context_codec_schema_versions.rs. Change only register_context_with_owner generic context schema guard from version!=1 to version==0. register_context delegates to this method and therefore accepts the same nonzero u32 domain. Keys must remain nonempty trimmed, duplicate stable keys and duplicate Rust types still reject with IncompatibleRegistration before inserting either map. u32::MAX is a valid caller version. No codec migration, autoupgrade, fallback or simultaneous two schemas for one type/key registry. Caller chooses exact trusted schema registry at capture/restore.

Restart-template registration version!=1 at checkpoint.rs735 remains unchanged. Flow native image versions, component-store schema1 and wire schema/native tags remain unchanged; caller context version is an independent opaque manifest u32. No public struct/error/signature/ABI/Arrow changes.

## Source-backed propagation and validation

ContextCodecImpl capture writes self.version into FlowContextStoreV1. Capture registration records write codec.version at1081. Native restore checks image.contexts entry.version equals trusted codec.version at3513; contextstore version equals codec.version at3530; concrete codec.restore repeats exact key/version check before callerdecoder at579. ComponentStore internal version1 at598 is unrelated and remains1. Wire context registration/store scanners/writers encode u32 opaque version and do not restrict1; V1/V2/V3/V4 image formats already represent arbitrary u32. Ref accessors and C2outer consume those manifests without hardcodedcontext1 checks. Audit found no additional production context1 assumption beyond registrationguard; fixture schema1 registrations remain legitimate.

Zero version must reject. A nativeimage with registration/store schema2 restored using schema1 (or viceversa) fails IncompatibleRegistration before any decoder; neither image bytes nor nativeversion are reinterpreted. Codec byte payload version validation is caller-owned (e.g. TransitV2 header) and remains separate from registry manifest matching.

## Exact required runtime proof names

1. context_codec_nonzero_versions_roundtrip_actual_runtime: actual sameFlow workcontext captures/restores manifest2 and u32MAX via owner-aware and simple registration APIs; correct rowowner/currentrebind; actual timedwork grant/completion/context suffix equality. Captureexactversion asserted in both registration/store rows.
2. context_codec_zero_and_duplicate_registration_atomic: validbaseline; zero/simple/owner-aware reject, duplicatekey/differenttype and sametype/differentkey reject; subsequentvalidregistration/capture demonstrates no partialmap mutation, no callercodec invoked by registration.
3. context_codec_manifest_mismatch_rejects_before_decoder: registrationrowonly and storerowonly wrongversion; bothchanged schema2 image vs schema1 registry and reverse; decodercounter0 with positivebaselinecounter>0; sourceimage unchanged.
4. context_codec_legacy_wire_versions_preserve_schema_and_bytes: actual AdmissionOrder runtime withschema2 roundtrips nativeV1/V2/V3/V4 and directwire each underexistingformats; exactu32 manifest survives; existing literaloldbytes/schema1tests unchanged. No newliteralgolden needed for this additive registration guard.
5. context_codec_restart_schema_guard_remains_v1: actual restartversion2/zero reject under existing rules, version1 positive; contextschema2 success does not broaden restartpolicy.

Format first; pinned Rust/Cargo1.99 strictDESall-targetClippy, fullDESpackage tests all5names+existingliteraltests, twofileformat. Maximum3faileddevelopmentexecutions, everyfailure root-reviewSTOP; exactpairedreceipt/authority/finalhash evidence before acceptancecommit. Native gate sourceonly; combinedfourpackagequalification later validatesallconsumercompatibility.

## Integration sequencing and ABM failure history

Current ABM wire packet cannot edit protected DES dependency. Stop that packet truthfully at dependencyboundary, preserve failedreceipt/result/sourcecandidate and claimrelease. Accept this DES prerequisite in DESdevelopmentbranch before routing writer reserves checkpoint.rs. Integrate accepted prerequisite into ABM clean preservationworktree, preserve priorABMwirefailhistory, bind fresh actualbase/context/driver/authority before resuming fullABMproofs. Do not silently rebase activeclaimedsource or reset historicalfailurecount. DirectDESrouting follows acceptedprerequisite when sharedcheckpointownership free. ActualTransitV2codec remains schema2 and its finaltests prove externalintegration.

## Compatibility review

Additive hidden experimental Rust registration behavior: previouslyrejected positiveversions nowaccepted; version1unchanged, zero/restart/duplicates remainfailclosed. Exacttrustedmanifest checks prevent accidentalwrongdecoder; nonzeroversion is metadata, not trust/authentication. No release reviewerapproval or parentpin implied. Public API signatures/error types unchanged; engine stays generic and EDpolicy-free.


## Bound source excerpts (full source hashes in packet)

### crates/kairo-ecs-des/src/flow/checkpoint.rs:500-620
500: 
501: struct ContextCodecImpl<C> {
502:     key: String,
503:     version: u32,
504:     encode: OwnedEncode<C>,
505:     decode: OwnedDecode<C>,
506: }
507: 
508: impl<C: 'static> ContextCodec for ContextCodecImpl<C> {
509:     fn key(&self) -> &str {
510:         &self.key
511:     }
512:     fn version(&self) -> u32 {
513:         self.version
514:     }
515:     fn context_type(&self) -> TypeId {
516:         TypeId::of::<C>()
517:     }
518:     fn store_type(&self) -> TypeId {
519:         TypeId::of::<WorkContext<C>>()
520:     }
521:     fn shape(
522:         &self,
523:         registry: &ComponentRegistry,
524:         limits: ComponentCheckpointLimits,
525:     ) -> Result<Option<(usize, usize)>, FlowCheckpointError> {
526:         let Some(store) = registry.store::<WorkContext<C>>() else {
527:             return Ok(None);
528:         };
529:         let shape = store.checkpoint_dimensions();
530:         if shape.0 > limits.max_rows {
531:             return Err(FlowCheckpointError::Component(
532:                 ComponentCheckpointStructureError::RowLimitExceeded,
533:             ));
534:         }
535:         if shape.1 > limits.max_sparse_slots {
536:             return Err(FlowCheckpointError::Component(
537:                 ComponentCheckpointStructureError::SparseSlotLimitExceeded,
538:             ));
539:         }
540:         Ok(Some(shape))
541:     }
542:     fn capture(
543:         &self,
544:         registry: &ComponentRegistry,
545:         limits: ComponentCheckpointLimits,
546:         remaining_bytes: &mut usize,
547:     ) -> Result<Option<FlowContextStoreV1>, FlowCheckpointError> {
548:         let Some(store) = registry.store::<WorkContext<C>>() else {
549:             return Ok(None);
550:         };
551:         let image = store
552:             .checkpoint_state_with(limits, |context| {
553:                 let bytes = (self.encode)(&context.0, *remaining_bytes)
554:                     .map_err(ComponentStoreCodecError)?;
555:                 if bytes.len() > *remaining_bytes {
556:                     return Err(ComponentStoreCodecError(FlowCheckpointCodecError(
557:                         "codec exceeded its remaining byte budget".to_owned(),
558:                     )));
559:                 }
560:                 *remaining_bytes -= bytes.len();
561:                 Ok(bytes)
562:             })
563:             .map_err(|error| match error {
564:                 ComponentCheckpointError::Structure(error) => FlowCheckpointError::Component(error),
565:                 ComponentCheckpointError::Codec(ComponentStoreCodecError(error)) => {
566:                     FlowCheckpointError::Codec {
567:                         key: self.key.clone(),
568:                         error,
569:                     }
570:                 }
571:             })?;
572:         Ok(Some(FlowContextStoreV1 {
573:             codec_key: self.key.clone(),
574:             version: self.version,
575:             sparse_slots: image.sparse_slots,
576:             rows: image.rows,
577:         }))
578:     }
579:     fn restore(
580:         &self,
581:         registry: &mut ComponentRegistry,
582:         image: FlowContextStoreV1,
583:         rebind: &FlowCheckpointRebindV1,
584:         limits: ComponentCheckpointLimits,
585:     ) -> Result<(), FlowCheckpointError> {
586:         if image.codec_key != self.key || image.version != self.version {
587:             return Err(FlowCheckpointError::IncompatibleRegistration(
588:                 self.key.clone(),
589:             ));
590:         }
591:         let mut rows = Vec::new();
592:         rows.try_reserve_exact(image.rows.len())
593:             .map_err(|_| FlowCheckpointError::AllocationFailed)?;
594:         for (owner, bytes) in image.rows {
595:             rows.push((owner, (owner, bytes)));
596:         }
597:         let payload = ComponentStoreCheckpointV1 {
598:             version: 1,
599:             sparse_slots: image.sparse_slots,
600:             rows,
601:         };
602:         let store =
603:             ComponentStore::from_checkpoint_state_with(payload, limits, |(owner, bytes)| {
604:                 (self.decode)(&bytes, owner, rebind)
605:                     .map(WorkContext)
606:                     .map_err(ComponentStoreCodecError)
607:             })
608:             .map_err(|error| match error {
609:                 ComponentCheckpointError::Structure(error) => FlowCheckpointError::Component(error),
610:                 ComponentCheckpointError::Codec(ComponentStoreCodecError(error)) => {
611:                     FlowCheckpointError::Codec {
612:                         key: self.key.clone(),
613:                         error,
614:                     }
615:                 }
616:             })?;
617:         registry.register::<WorkContext<C>>();
618:         *registry
619:             .store_mut::<WorkContext<C>>()
620:             .expect("registered context store") = store;

### crates/kairo-ecs-des/src/flow/checkpoint.rs:650-752
650:     pub fn new() -> Self {
651:         Self::default()
652:     }
653: 
654:     /// Register one owned context codec for its Rust type. Multiple Flow
655:     /// registration names of that type share this single physical store.
656:     pub fn register_context<C: 'static>(
657:         &mut self,
658:         codec_key: impl Into<String>,
659:         version: u32,
660:         encode: EncodeContext<C>,
661:         decode: DecodeContext<C>,
662:     ) -> Result<(), FlowCheckpointError> {
663:         self.register_context_with_owner(codec_key, version, encode, move |bytes, _, view| {
664:             decode(bytes, view)
665:         })
666:     }
667: 
668:     /// Register caller-owned immutable codec environments. The outer trusted
669:     /// model/configuration manifest binds their meaning to the stable codec key.
670:     /// The decoder receives the actual dense row owner and a read-only view;
671:     /// returned values must own their state and must not alias mutable outsiders.
672:     pub fn register_context_with_owner<C: 'static>(
673:         &mut self,
674:         codec_key: impl Into<String>,
675:         version: u32,
676:         encode: impl Fn(&C, usize) -> Result<Vec<u8>, FlowCheckpointCodecError> + 'static,
677:         decode: impl Fn(&[u8], EntityId, &FlowCheckpointRebindV1) -> Result<C, FlowCheckpointCodecError>
678:             + 'static,
679:     ) -> Result<(), FlowCheckpointError> {
680:         let key = codec_key.into();
681:         if key.trim().is_empty()
682:             || version != 1
683:             || self.contexts.contains_key(&key)
684:             || self.context_types.contains_key(&TypeId::of::<C>())
685:         {
686:             return Err(FlowCheckpointError::IncompatibleRegistration(key));
687:         }
688:         self.context_types.insert(TypeId::of::<C>(), key.clone());
689:         self.contexts.insert(
690:             key.clone(),
691:             Box::new(ContextCodecImpl::<C> {
692:                 key,
693:                 version,
694:                 encode: Box::new(encode),
695:                 decode: Box::new(decode),
696:             }),
697:         );
698:         Ok(())
699:     }
700: 
701:     fn context_for_type(&self, context_type: TypeId) -> Option<&dyn ContextCodec> {
702:         let key = self.context_types.get(&context_type)?;
703:         self.contexts.get(key).map(Box::as_ref)
704:     }
705: 
706:     /// Register the versioned owned template codec for one `(T, C)` pair.
707:     pub fn register_restart_template<T: 'static, C: 'static>(
708:         &mut self,
709:         codec_key: impl Into<String>,
710:         version: u32,
711:         encode: fn(&T, usize) -> Result<Vec<u8>, FlowCheckpointCodecError>,
712:         decode: fn(&[u8], &FlowCheckpointRebindV1) -> Result<T, FlowCheckpointCodecError>,
713:     ) -> Result<(), FlowCheckpointError> {
714:         self.register_restart_template_with_owner::<T, C>(
715:             codec_key,
716:             version,
717:             encode,
718:             move |bytes, _, view| decode(bytes, view),
719:         )
720:     }
721: 
722:     /// Register caller-owned template codec environments under the same trusted
723:     /// manifest contract as context codecs. Factory bindings remain explicit.
724:     pub fn register_restart_template_with_owner<T: 'static, C: 'static>(
725:         &mut self,
726:         codec_key: impl Into<String>,
727:         version: u32,
728:         encode: impl Fn(&T, usize) -> Result<Vec<u8>, FlowCheckpointCodecError> + 'static,
729:         decode: impl Fn(&[u8], EntityId, &FlowCheckpointRebindV1) -> Result<T, FlowCheckpointCodecError>
730:             + 'static,
731:     ) -> Result<(), FlowCheckpointError> {
732:         let key = codec_key.into();
733:         let types = (TypeId::of::<T>(), TypeId::of::<C>());
734:         if key.trim().is_empty()
735:             || version != 1
736:             || self.templates.contains_key(&key)
737:             || self.template_types.contains_key(&types)
738:         {
739:             return Err(FlowCheckpointError::IncompatibleRegistration(key));
740:         }
741:         self.template_types.insert(types, key.clone());
742:         self.templates.insert(
743:             key.clone(),
744:             Box::new(RestartCodecImpl::<T, C> {
745:                 key,
746:                 version,
747:                 encode: Box::new(encode),
748:                 decode: Box::new(decode),
749:                 factories: BTreeMap::new(),
750:             }),
751:         );
752:         Ok(())

### crates/kairo-ecs-des/src/flow/checkpoint.rs:3495-3550
3495:             if key_bytes > limits.max_key_bytes {
3496:                 return Err(FlowCheckpointError::LimitExceeded("key bytes"));
3497:             }
3498:             Ok(())
3499:         };
3500:         let mut seen_context_names = BTreeSet::new();
3501:         for entry in &image.contexts {
3502:             add_key(&entry.runtime_key)?;
3503:             add_key(&entry.codec_key)?;
3504:             if !seen_context_names.insert(entry.runtime_key.clone()) {
3505:                 return Err(FlowCheckpointError::InvalidState(
3506:                     "duplicate context registration",
3507:                 ));
3508:             }
3509:             let codec = codecs
3510:                 .contexts
3511:                 .get(&entry.codec_key)
3512:                 .ok_or_else(|| FlowCheckpointError::MissingContextCodec(entry.codec_key.clone()))?;
3513:             if codec.version() != entry.version {
3514:                 return Err(FlowCheckpointError::IncompatibleRegistration(
3515:                     entry.codec_key.clone(),
3516:                 ));
3517:             }
3518:         }
3519:         let mut context_payloads = BTreeMap::new();
3520:         for store in &image.context_stores {
3521:             add_key(&store.codec_key)?;
3522:             if context_payloads.contains_key(&store.codec_key) {
3523:                 return Err(FlowCheckpointError::InvalidState("duplicate context store"));
3524:             }
3525:             let codec = codecs
3526:                 .contexts
3527:                 .get(&store.codec_key)
3528:                 .ok_or_else(|| FlowCheckpointError::MissingContextCodec(store.codec_key.clone()))?;
3529:             validate_rows(store.sparse_slots, &store.rows, component_limits)?;
3530:             if store.version != codec.version() {
3531:                 return Err(FlowCheckpointError::IncompatibleRegistration(
3532:                     store.codec_key.clone(),
3533:                 ));
3534:             }
3535:             for (_, bytes) in &store.rows {
3536:                 payload_bytes = payload_bytes
3537:                     .checked_add(bytes.len())
3538:                     .ok_or(FlowCheckpointError::LimitExceeded("payload bytes"))?;
3539:             }
3540:             context_payloads.insert(
3541:                 store.codec_key.clone(),
3542:                 (store.sparse_slots, store.rows.len()),
3543:             );
3544:         }
3545:         let expected_context_stores: BTreeSet<_> = image
3546:             .contexts
3547:             .iter()
3548:             .map(|entry| entry.codec_key.as_str())
3549:             .collect();
3550:         if expected_context_stores.len() != context_payloads.len()

### crates/kairo-ecs-des/tests/flow_checkpoint_v1.rs:1080-1125
1080: #[test]
1081: fn schema_codec_key_index_and_payload_limits_fail_before_decoder() {
1082:     let codecs = register_codec(mutate_context);
1083:     let mut source = FlowRuntime::new();
1084:     source
1085:         .register_domain_view_hook("model.ctx", EventKind::custom(91), mutate_context)
1086:         .unwrap();
1087:     let actor = source.spawn_actor().unwrap();
1088:     source
1089:         .create_actor_domain_context(
1090:             actor,
1091:             "model.ctx",
1092:             EventKind::custom(91),
1093:             Context { value: 17 },
1094:         )
1095:         .unwrap();
1096:     let base = source
1097:         .capture_checkpoint(&codecs, FlowCheckpointLimits::default())
1098:         .unwrap();
1099: 
1100:     let mut bad_schema = base.clone();
1101:     bad_schema.context_stores[0].version = 2;
1102:     let mut bad_key = base.clone();
1103:     bad_key.context_stores[0].codec_key = "unknown.codec".into();
1104:     let mut bad_index = base.clone();
1105:     bad_index.context_stores[0].sparse_slots = 0;
1106:     let mut too_many_actor_domains = base.clone();
1107:     let duplicate_domain = too_many_actor_domains.actor_domains[0];
1108:     too_many_actor_domains.actor_domains.push(duplicate_domain);
1109: 
1110:     for image in [bad_schema, bad_key, bad_index] {
1111:         DECODE_CALLS.with(|count| count.set(0));
1112:         assert!(
1113:             FlowRuntime::restore_checkpoint(image, &codecs, FlowCheckpointLimits::default())
1114:                 .is_err()
1115:         );
1116:         DECODE_CALLS.with(|count| assert_eq!(count.get(), 0));
1117:     }
1118:     DECODE_CALLS.with(|count| count.set(0));
1119:     assert!(matches!(
1120:         FlowRuntime::restore_checkpoint(
1121:             too_many_actor_domains,
1122:             &codecs,
1123:             FlowCheckpointLimits {
1124:                 max_actors: 1,
1125:                 ..FlowCheckpointLimits::default()
