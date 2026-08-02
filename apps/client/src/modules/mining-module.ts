import { BaseModule } from './base-module';
import type {
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ResearchDefinition,
  ShipModule,
} from './types';

export class MiningModule extends BaseModule {
  readonly type = 'mining';
  readonly name = 'Mining module';
  readonly baseDurability = 2_000;
  readonly research: ResearchDefinition = {
    module: 'mining',
    name: this.name,
    cost: { iron: 0, silicates: 0, ice: 0 },
  };
  readonly attributes: readonly ModuleAttributeDefinition[] = [
    {
      attribute: 'efficiency',
      label: 'Efficiency',
      defaultValue: 2,
      materials: { iron: 20, silicates: 8 },
    },
    {
      attribute: 'durability',
      label: 'Durability',
      defaultValue: this.baseDurability,
      materials: { iron: 14, silicates: 12, ice: 4 },
    },
    {
      attribute: 'range',
      label: 'Range',
      defaultValue: 50_000,
      materials: { iron: 18, silicates: 10, ice: 6 },
    },
  ] as const;

  getDurabilityDrainRate(module: ShipModule): ModuleDurabilityDrainRate {
    return {
      amount: this.getRateKgPerSecond(module.levels),
      unit: 'second',
    };
  }

  getRateKgPerSecond(levels: ShipModule['levels']) {
    return this.getAttributeValue(levels, 'efficiency');
  }

  getRangeMeters(levels: ShipModule['levels']) {
    return this.getAttributeValue(levels, 'range');
  }
}
