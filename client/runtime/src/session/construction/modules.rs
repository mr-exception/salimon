//! Physical optional equipment: funded maintenance, durable downtime and commissioning.
use super::*;
use salimon_ship::structure::{construction::Work, funding::Funding};
use salimon_world::resources::ResourceFragment;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Operation {
    Upgrade,
    Restore,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Maintenance {
    pub kind: Operation,
    pub funding: Funding,
    pub work: Work,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::session) struct Module {
    pub(super) tier: u8,
    /// Activated maintenance never restores availability merely by cancelling.
    pub(super) unavailable: bool,
    pub(super) fault: Option<Fault>,
    pub(super) installed: Vec<ResourceFragment>,
    pub(super) maintenance: Option<Maintenance>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Fault {
    Damage,
    NoPower,
    Connection,
    Clearance,
}

fn blueprint(kind: Operation) -> Blueprint {
    let mut b = registered_catalog()
        .find("utility-module")
        .expect("registered utility blueprint")
        .clone();
    let (mg, seconds) = match kind {
        Operation::Upgrade => (2_000_000, 90),
        Operation::Restore => (1_000_000, 30),
    };
    b.requirements[0].milligrams = mg;
    b.duration_seconds = seconds;
    b
}
impl Module {
    pub(in crate::session) fn receipts(&self) -> impl Iterator<Item = &ResourceFragment> {
        self.installed.iter().chain(
            self.maintenance
                .iter()
                .flat_map(|m| m.funding.receipts.iter()),
        )
    }
    fn available(&self, p: &Placeholder) -> bool {
        !p.work.destroyed
            && self.tier > 0
            && !self.unavailable
            && self.fault.is_none()
            && !p.work.damaged
    }
}
impl ConstructionSave {
    pub(super) fn validate_modules(&self) -> Result<(), salimon_persistence::SaveError> {
        use salimon_persistence::invalid;

        for (id, m) in &self.modules {
            let p = self
                .projects
                .inspect(*id)
                .ok_or_else(|| invalid("module site missing"))?;
            if p.blueprint.id != "utility-module"
                || m.tier > 2
                || m.installed.len() > 128
                || (!p.work.completed && (m.tier > 0 || m.maintenance.is_some()))
                || (m.tier == 0 && (!m.installed.is_empty() || m.maintenance.is_some()))
                || (p.work.completed && m.tier == 0 && m.fault.is_none())
                || (m.fault.is_some() && !m.unavailable)
            {
                return Err(invalid("module identity/tier/commission state"));
            }
            let mut installed_mg = 0;
            for f in &m.installed {
                if f.material().resource() != p.blueprint.requirements[0].resource {
                    return Err(invalid("module installed material"));
                }
                installed_mg += salimon_ship::structure::funding::milligrams(*f)
                    .map_err(|_| invalid("module installed receipt"))?;
            }
            if installed_mg > 0 {
                Funding {
                    receipts: m.installed.clone(),
                    progress_ppm: 0,
                }
                .validate(&[Requirement {
                    resource: p.blueprint.requirements[0].resource,
                    milligrams: installed_mg,
                }])
                .map_err(|_| invalid("invalid installed module receipt"))?;
            }
            if (installed_mg % 1_000_000 != 0) || (m.tier == 2 && installed_mg < 2_000_000) {
                return Err(invalid("module installed mass/tier"));
            }
            if let Some(op) = &m.maintenance {
                let b = blueprint(op.kind);
                op.funding
                    .validate(&b.requirements)
                    .map_err(|_| invalid("module funding"))?;
                op.work
                    .validate(&op.funding, &b)
                    .map_err(|_| invalid("module work"))?;
                if m.installed.len() + op.funding.receipts.len() > 128
                    || m.tier == 0
                    || (op.kind == Operation::Upgrade && m.tier != 1)
                    || ((op.funding.funded(&b.requirements) || op.work.elapsed_nanos.is_some())
                        && !m.unavailable)
                    || (op.kind == Operation::Restore && !m.unavailable)
                    || (op.work.completed && m.fault.is_none())
                {
                    return Err(invalid("module maintenance phase"));
                }
            }
        }
        if self.projects.jobs().any(|p| {
            p.work.completed
                && p.blueprint.id == "utility-module"
                && !self.modules.contains_key(&p.id)
        }) {
            return Err(invalid("commissioned module source missing"));
        }
        Ok(())
    }
}
impl GameplaySession {
    /// One bus-connected optional processor is sufficient. Additional units do
    /// not stack throughput; watts and joules remain owned by Recovery/Core.
    pub(in crate::session) fn utility_recovery_multiplier(&self) -> f64 {
        self.building
            .source
            .modules
            .iter()
            .filter_map(|(id, m)| {
                let p = self.building.source.projects.inspect(*id)?;
                m.available(p).then_some(match m.tier {
                    1 => 1.25_f64,
                    2 => 1.5,
                    _ => 1.,
                })
            })
            .fold(1., f64::max)
    }
    pub(super) fn module_fault(&self, id: PlaceholderId) -> Option<Fault> {
        let p = self.building.source.projects.inspect(id)?;
        let m = self.building.source.modules.get(&id);
        if p.work.damaged
            || m.and_then(|m| m.maintenance.as_ref())
                .is_some_and(|op| op.work.damaged)
        {
            return Some(Fault::Damage);
        }
        if !self.ship.snapshot().power_available {
            return Some(Fault::NoPower);
        }
        if !self.construction_hull_ready()
            || !(has_physical_support(p.blueprint.footprint, p.space)
                || installed::supported_by(&self.building.structure, p.space))
        {
            return Some(Fault::Connection);
        }
        // Check current real occupancy while excluding this installed module's
        // own geometry. Its reservation still protects other placements.
        let own = [
            p.space.min[0],
            p.space.max[0],
            p.space.min[1],
            p.space.max[1],
            p.space.min[2],
            p.space.max[2],
        ];
        let mut obstacles = self.construction_obstacles(p.blueprint.footprint);
        obstacles.retain(|o| o.space != envelope(own));
        if obstacles.iter().any(|o| {
            (0..3).all(|i| p.space.min[i] < o.space.max[i] && p.space.max[i] > o.space.min[i])
        }) {
            return Some(Fault::Clearance);
        }
        None
    }
    fn commission_module(&mut self, id: PlaceholderId) {
        let fault = self.module_fault(id);
        let m = self.building.source.modules.entry(id).or_default();
        m.fault = fault;
        if fault.is_some() {
            m.unavailable = true;
            return;
        }
        if let Some(op) = m.maintenance.take() {
            assert!(op.work.completed, "only fully built modules commission");
            if op.kind == Operation::Upgrade {
                m.tier = 2;
            }
            m.installed.extend(op.funding.receipts);
        } else if m.tier == 0 {
            m.tier = 1;
        }
        m.unavailable = false;
    }
    pub(super) fn module_action(&mut self) {
        let Some(id) = self.aimed_construction() else {
            return;
        };
        let Some(p) = self.building.source.projects.inspect(id) else {
            return;
        };
        if p.blueprint.id != "utility-module" || !p.work.completed {
            return;
        }
        let m = self
            .building
            .source
            .modules
            .get(&id)
            .expect("completed module source");
        if m.maintenance.as_ref().is_some_and(|op| !op.work.completed) {
            self.feedback = Some("Maintenance planned: E feed, T stop/resume, RMB cancel");
            return;
        }
        if m.fault.is_some() {
            if !self
                .building
                .source
                .projects
                .inspect(id)
                .unwrap()
                .work
                .completed
            {
                return;
            }
            self.commission_module(id);
            self.feedback =
                Some("Commissioning checked; clear the reported fault and press G to retry");
            return;
        }
        let kind = if m.unavailable {
            Operation::Restore
        } else if m.tier == 1 {
            Operation::Upgrade
        } else {
            self.feedback = Some("Utility processor is at maximum tier");
            return;
        };
        if m.installed.len() > 120 {
            self.feedback = Some("Module maintenance receipt capacity reached");
            return;
        }
        self.building
            .source
            .modules
            .get_mut(&id)
            .unwrap()
            .maintenance = Some(Maintenance {
            kind,
            funding: Default::default(),
            work: Default::default(),
        });
        self.feedback = Some(match kind {
            Operation::Upgrade => "Upgrade planned: E feed 2 kg iron; 90s at 3 kW",
            Operation::Restore => "Restoration planned: E feed 1 kg iron; 30s at 3 kW",
        });
    }
    pub(super) fn feed_module(&mut self, id: PlaceholderId) -> bool {
        let Some(m) = self.building.source.modules.get_mut(&id) else {
            return false;
        };
        let Some(op) = &mut m.maintenance else {
            return false;
        };
        if m.installed.len() + op.funding.receipts.len() >= 128 {
            self.feedback = Some("Maintenance material receipt capacity reached");
            return true;
        }
        if op.work.completed {
            return false;
        }
        let b = blueprint(op.kind);
        self.feedback = Some(
            if op
                .funding
                .contribute(
                    &mut self.mining.session,
                    &b.requirements,
                    self.building.catalog.approved_economy,
                )
                .is_ok()
            {
                if op.funding.funded(&b.requirements) {
                    m.unavailable = true;
                }
                "Maintenance material accepted; activated work disables the whole module"
            } else {
                "Maintenance needs a fitting whole-milligram iron fragment"
            },
        );
        self.sync_carried();
        self.sync_fabricator_mass();
        true
    }
    pub(super) fn cancel_module(&mut self, id: PlaceholderId) -> Result<(), ()> {
        let m = self.building.source.modules.get(&id).ok_or(())?;
        let op = m.maintenance.as_ref().ok_or(())?;
        let b = blueprint(op.kind);
        let refunds = op.funding.refunds(&b.requirements).map_err(|_| ())?;
        self.return_construction_materials(refunds)?;
        let m = self.building.source.modules.get_mut(&id).unwrap();
        m.maintenance = None;
        m.fault = None;
        // The source tier/installed receipts remain real but unavailable until
        // a separate funded restoration succeeds. No cancellation rollback.
        self.sync_carried();
        self.sync_fabricator_mass();
        Ok(())
    }
    pub(super) fn module_stop_resume(&mut self, id: PlaceholderId) -> bool {
        let Some(op) = self
            .building
            .source
            .modules
            .get_mut(&id)
            .and_then(|m| m.maintenance.as_mut())
        else {
            return false;
        };
        if !op.work.completed {
            op.work.stopped = !op.work.stopped;
        }
        self.feedback = Some("Maintenance stop latch changed; activation downtime is retained");
        true
    }
    pub(super) fn module_blueprint(&self, id: PlaceholderId) -> Option<Blueprint> {
        self.building
            .source
            .modules
            .get(&id)?
            .maintenance
            .as_ref()
            .map(|op| blueprint(op.kind))
    }
    pub(super) fn module_status(&self, id: PlaceholderId) -> Option<String> {
        let m = self.building.source.modules.get(&id)?;
        if m.tier == 0 && !self.building.source.projects.inspect(id)?.work.completed {
            return None;
        }
        Some(if let Some(op) = &m.maintenance {
            format!(
                "{:?} {:.1}% | {} | E feed / T stop / RMB cancel",
                op.kind,
                f64::from(op.funding.progress_ppm) / 10_000.,
                if let Some(f) = m.fault {
                    format!("FAULT {f:?}; G retry")
                } else if op.work.damaged {
                    "DAMAGED".into()
                } else if op.work.stopped {
                    "STOPPED".into()
                } else if m.unavailable {
                    "OFFLINE".into()
                } else {
                    "ONLINE until funded".into()
                }
            )
        } else if let Some(f) = m.fault {
            format!("COMMISSION FAULT {f:?} | G retry")
        } else if m.unavailable {
            format!("T{} OFFLINE | G plan physical restoration", m.tier)
        } else {
            format!("UTILITY PROCESSOR T{} ONLINE | G upgrade", m.tier)
        })
    }
    pub(in crate::session) fn module_checkpoint(&self) -> Vec<u8> {
        let values: Vec<_> = self
            .building
            .source
            .modules
            .iter()
            .map(|(id, m)| {
                (
                    id,
                    m.tier,
                    m.unavailable,
                    m.fault,
                    m.installed.iter().map(|f| f.id()).collect::<Vec<_>>(),
                    m.maintenance.as_ref().map(|o| {
                        (
                            o.kind,
                            o.work.completed,
                            o.work.damaged,
                            o.work.stopped,
                            o.funding
                                .receipts
                                .iter()
                                .map(|f| f.id())
                                .collect::<Vec<_>>(),
                        )
                    }),
                )
            })
            .collect();
        serde_json::to_vec(&values).expect("integer module event source")
    }
    pub(super) fn advance_modules(&mut self, delta: Duration) {
        let ids: Vec<_> = self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| p.blueprint.id == "utility-module")
            .map(|p| p.id)
            .collect();
        for id in ids {
            let p = self.building.source.projects.inspect(id).unwrap();
            let m = self.building.source.modules.entry(id).or_default();
            if !p.work.completed || p.work.destroyed {
                continue;
            }
            if m.tier == 0 && m.fault.is_none() {
                self.commission_module(id);
                continue;
            }
            let Some(op) = &m.maintenance else {
                continue;
            };
            let b = blueprint(op.kind);
            if op.work.stopped
                || op.work.damaged
                || op.work.completed
                || !op.funding.funded(&b.requirements)
            {
                continue;
            }
            if self.module_fault(id).is_some() {
                continue;
            }
            let op = self.building.source.modules[&id]
                .maintenance
                .as_ref()
                .unwrap();
            let nanos = delta
                .as_nanos()
                .min(op.work.remaining(&op.funding, &b).as_nanos()) as u64;
            let requested = nanos as f64 / 1e9 * 3_000.;
            if nanos == 0 {
                continue;
            }
            let funded = self
                .ship
                .fund_auxiliary_work(requested, self.recovery_reserve(), delta)
                .expect("bounded module request");
            let paid = if funded == requested {
                nanos
            } else {
                ((funded / 3_000. * 1e9).floor() as u64).min(nanos)
            };
            if paid == 0 {
                continue;
            }
            let m = self.building.source.modules.get_mut(&id).unwrap();
            m.unavailable = true;
            let op = m.maintenance.as_mut().unwrap();
            op.work
                .advance(&mut op.funding, &b, Duration::from_nanos(paid))
                .expect("validated paid maintenance");
            if op.work.completed {
                self.commission_module(id);
            }
        }
    }
}

#[cfg(test)]
mod tests;
