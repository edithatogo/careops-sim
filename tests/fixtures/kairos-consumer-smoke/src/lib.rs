use kairo_ecs_abm::ABMContext;
use kairo_ecs_core::Scheduler;
use kairo_ecs_des::Resource;

pub fn exercise_public_apis() -> (usize, bool, u64, usize) {
    let scheduler = Scheduler::new();
    let mut model = ABMContext::new(0xD1_01);
    let patient = model.spawn_agent();
    let mut bed = Resource::new("bed", 1);
    let acquired = bed.request(patient);
    (
        scheduler.pending_events(),
        acquired,
        bed.available_count(),
        bed.queue_length(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn downstream_consumer_can_use_core_des_and_abm_apis() {
        assert_eq!(super::exercise_public_apis(), (0, true, 0, 0));
    }
}
