import { BaseModule } from './base-module';
import type {
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ModuleLevels,
  ResearchDefinition,
} from './types';

export class EnergyCoreModule extends BaseModule {
  readonly type = 'energy-core';
  readonly name = 'Energy Core module';
  readonly baseDurability = 800;
  readonly durabilityDrainPerRefuel = 1;
  readonly research: ResearchDefinition = {
    module: 'energy-core',
    name: this.name,
    cost: { iron: 140, silicates: 100, carbon: 50 },
  };
  readonly attributes: readonly ModuleAttributeDefinition[] = [
    {
      attribute: 'capacity',
      label: 'Capacity',
      defaultValue: 1_000_000_000,
      materials: { iron: 38, silicates: 28, carbon: 14 },
    },
  ] as const;

  getMaxDurability() {
    return this.baseDurability;
  }

  getDurabilityDrainRate(): ModuleDurabilityDrainRate {
    return {
      amount: this.durabilityDrainPerRefuel,
      unit: 'refuel',
    };
  }

  getCapacityKns(levels: ModuleLevels) {
    return this.getAttributeValue(levels, 'capacity') / 1_000;
  }
}
