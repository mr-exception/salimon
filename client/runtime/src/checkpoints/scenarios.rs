//! Real launch/worker/restart transactions over a fault-injected storage boundary.
use super::*;
use salimon_survival::{Command, Points};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug)]
enum Fault {
    Before,
    Torn,
    After,
}
#[derive(Default)]
struct Disk {
    slots: [Option<Vec<u8>>; 2],
    archives: BTreeMap<(u32, u64), Vec<u8>>,
    operations: usize,
    fail: Option<(usize, Fault)>,
}
impl Disk {
    fn fault(&mut self) -> Option<Fault> {
        self.operations += 1;
        self.fail
            .filter(|(at, _)| *at == self.operations)
            .map(|(_, fault)| fault)
    }
}
#[derive(Clone, Default)]
struct Store(Arc<Mutex<Disk>>);
impl CheckpointStore for Store {
    fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError> {
        Ok(self.0.lock().unwrap().slots[slot].clone())
    }
    fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError> {
        let mut disk = self.0.lock().unwrap();
        let fault = disk.fault();
        if !matches!(fault, Some(Fault::Before)) {
            disk.slots[slot] = Some(if matches!(fault, Some(Fault::Torn)) {
                bytes[..bytes.len() / 2].to_vec()
            } else {
                bytes.to_vec()
            });
        }
        if fault.is_some() {
            Err(salimon_persistence::invalid(
                "injected checkpoint failure / disk full",
            ))
        } else {
            Ok(())
        }
    }
    fn archive_run(&mut self, version: u32, serial: u64, bytes: &[u8]) -> Result<(), SaveError> {
        let mut disk = self.0.lock().unwrap();
        let fault = disk.fault();
        if !matches!(fault, Some(Fault::Before | Fault::Torn)) {
            if let Some(previous) = disk.archives.get(&(version, serial)) {
                if previous != bytes {
                    return Err(salimon_persistence::invalid("conflicting archive"));
                }
            } else {
                disk.archives.insert((version, serial), bytes.to_vec());
            }
        }
        if fault.is_some() {
            Err(salimon_persistence::invalid("injected archive failure"))
        } else {
            Ok(())
        }
    }
    fn next_run_serial(&self) -> Result<u64, SaveError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .archives
            .keys()
            .map(|(_, serial)| serial + 1)
            .max()
            .unwrap_or(1))
    }
}
fn arm(store: &Store, offset: usize, fault: Fault) {
    let mut disk = store.0.lock().unwrap();
    disk.fail = Some((disk.operations + offset, fault));
}
fn disarm(store: &Store) {
    store.0.lock().unwrap().fail = None;
}
fn recover(store: &Store) -> GameplaySession {
    let mut manager = SaveManager::new(store.clone());
    let bytes = manager.recover(validate).unwrap().bytes.unwrap();
    let mut s = GameplaySession::default();
    s.restore_bytes(&bytes).unwrap();
    s
}
fn drain(host: &mut Checkpoints, s: &GameplaySession) {
    while host.busy {
        let result = host.results.recv_timeout(Duration::from_secs(5)).unwrap();
        host.completed(result, s);
        host.poll(s);
    }
}

#[test]
fn persistence_scenario_failed_manual_save_never_acknowledges_and_previous_full_session_recovers() {
    for fault in [Fault::Before, Fault::Torn, Fault::After] {
        let store = Store::default();
        let mut s = GameplaySession::new_run();
        let mut host = Checkpoints::with_store_at(store.clone(), &mut s, 100, false).unwrap();
        s.toggle_pause();
        let previous = s.consumables.clone();
        // A real inventory change makes recovering the wrong generation detectable.
        s.toggle_pause();
        s.command(crate::session::Command::Wheel(
            crate::session::consumable_wheel::WheelAction::Hold(true),
        ));
        s.command(crate::session::Command::Wheel(
            crate::session::consumable_wheel::WheelAction::Hover(Some(
                salimon_fabrication::ConsumableSlot::FoodOne,
            )),
        ));
        s.command(crate::session::Command::Wheel(
            crate::session::consumable_wheel::WheelAction::Primary(true),
        ));
        s.toggle_pause();
        assert_ne!(s.consumables, previous);
        let frozen = s.save_bytes(100).unwrap();
        arm(&store, 1, fault);
        host.request(control::Request::SaveAndExit, &s);
        drain(&mut host, &s);
        assert_eq!(host.status().0, control::Status::Failed, "{fault:?}");
        assert!(!host.status().2);
        assert!(host.status().1.is_none());
        assert_eq!(s.save_bytes(100).unwrap(), frozen);
        drop(host); // Crash before any retry or clean-exit checkpoint.
        let loaded = recover(&store);
        if matches!(fault, Fault::After) {
            assert_eq!(loaded.consumables, s.consumables);
        } else {
            assert_eq!(loaded.consumables, previous);
        }
        disarm(&store);
        let mut loaded = GameplaySession::default();
        let host = Checkpoints::with_store_at(store.clone(), &mut loaded, 100, false).unwrap();
        assert!(!loaded.is_paused() && !loaded.wheel.open && !loaded.wheel.primary_down);
        assert_eq!(host.status().0, control::Status::Idle);
        drop(host);
    }
}

#[test]
fn persistence_scenario_death_restart_failure_at_every_transaction_boundary() {
    for fault in [Fault::Before, Fault::Torn, Fault::After] {
        for offset in 1..=5 {
            // Two terminal copies, archive, two fresh copies.
            let store = Store::default();
            let mut s = GameplaySession::new_run();
            let mut host = Checkpoints::with_store_at(store.clone(), &mut s, 100, false).unwrap();
            s.survival.command(Command::Damage(Points::FULL)).unwrap();
            s.advance_automation(Duration::from_millis(16));
            // Start from a verified terminal checkpoint, as a normal death does.
            host.request(control::Request::SaveNow, &s);
            drain(&mut host, &s);
            let terminal = s.save_bytes(100).unwrap();
            let report = s.ended_run_bytes().unwrap();
            arm(&store, offset, fault);
            assert!(
                host.restart(&mut s).is_err(),
                "{fault:?} operation {offset}"
            );
            assert_eq!(s.save_bytes(100).unwrap(), terminal);
            drop(host);
            let recovered = recover(&store);
            assert!([1, 2].contains(&recovered.run.serial));
            if recovered.run.serial == 1 {
                assert!(recovered.survival.snapshot().terminal);
                assert_eq!(recovered.ended_run_bytes().unwrap(), report);
            } else {
                assert_eq!(
                    recovered.consumables,
                    GameplaySession::new_run().consumables
                );
                assert_eq!(recovered.elapsed, Duration::ZERO);
            }
            disarm(&store);
            let mut visible = GameplaySession::default();
            let mut host =
                Checkpoints::with_store_at(store.clone(), &mut visible, 100, false).unwrap();
            if visible.survival.snapshot().terminal {
                host.restart(&mut visible).unwrap();
            }
            assert_eq!(visible.run.serial, 2);
            assert_eq!(visible.survival.snapshot().health, Points::FULL);
            assert_eq!(store.0.lock().unwrap().archives.len(), 1);
            assert_eq!(
                store.0.lock().unwrap().archives.values().next().unwrap(),
                &report
            );
            drop(host);
            // Acknowledged restart retires both old generations.
            for damaged_slot in 0..2 {
                let saved = store.0.lock().unwrap().slots[damaged_slot].clone();
                store.0.lock().unwrap().slots[damaged_slot] = Some(b"corrupt".to_vec());
                assert_eq!(recover(&store).run.serial, 2);
                store.0.lock().unwrap().slots[damaged_slot] = saved;
            }
        }
    }
}
