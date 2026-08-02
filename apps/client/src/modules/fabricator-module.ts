import { BaseModule } from './base-module';
import type {
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ResearchDefinition,
} from './types';

export class FabricatorModule extends BaseModule {
  readonly type = 'fabricator';
  readonly name = 'Fabricator module';
  readonly baseDurability = 500;
  readonly durabilityDrainPerCraft = 1;
  readonly research: ResearchDefinition = {
    module: 'fabricator',
    name: this.name,
    cost: { iron: 80, silicates: 40, carbon: 20 },
  };
  readonly attributes: readonly ModuleAttributeDefinition[] = [
    {
      attribute: 'durability',
      label: 'Durability',
      defaultValue: this.baseDurability,
      materials: { iron: 28, silicates: 18, carbon: 10 },
    },
  ] as const;

  getDurabilityDrainRate(): ModuleDurabilityDrainRate {
    return {
      amount: this.durabilityDrainPerCraft,
      unit: 'craft',
    };
  }
}
