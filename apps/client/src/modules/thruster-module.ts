import { BaseModule } from './base-module';
import type {
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ModuleLevels,
  ResearchDefinition,
} from './types';

export class ThrusterModule extends BaseModule {
  readonly type = 'thruster';
  readonly name = 'Thruster module';
  readonly baseDurability = 100;
  readonly durabilityDrainPerSecond = 0.02;
  readonly research: ResearchDefinition = {
    module: 'thruster',
    name: this.name,
    cost: { iron: 120, silicates: 80, ice: 40 },
  };
  readonly attributes: readonly ModuleAttributeDefinition[] = [
    {
      attribute: 'power',
      label: 'Power',
      defaultValue: 1_000_000,
      materials: { iron: 45, silicates: 30, ice: 12 },
    },
    {
      attribute: 'durability',
      label: 'Durability',
      defaultValue: this.baseDurability,
      materials: { iron: 32, silicates: 34, ice: 8 },
    },
  ] as const;

  getDurabilityDrainRate(): ModuleDurabilityDrainRate {
    return {
      amount: this.durabilityDrainPerSecond,
      unit: 'second',
    };
  }

  getPowerKns(levels: ModuleLevels) {
    return this.getAttributeValue(levels, 'power');
  }
}
