import {
  DEFAULT_MODULE_UPGRADE_MULTIPLIER,
  multiplyInventoryCost,
} from './upgrade';
import type {
  Inventory,
  ModuleAttribute,
  ModuleAttributeDefinition,
  ModuleDurabilityDrainRate,
  ModuleGridCell,
  ModuleLevels,
  ModuleType,
  ResearchDefinition,
  ShipModule,
} from './types';

export abstract class BaseModule {
  abstract readonly type: ModuleType;
  abstract readonly name: string;
  abstract readonly attributes: readonly ModuleAttributeDefinition[];
  abstract readonly research: ResearchDefinition;
  abstract readonly baseDurability: number;

  create(id: string, position: ModuleGridCell): ShipModule {
    return {
      id,
      type: this.type,
      name: this.name,
      position,
      active: false,
      unlocked: true,
      levels: this.createDefaultLevels(),
      durability: this.baseDurability,
    };
  }

  getAttribute(attribute: ModuleAttribute) {
    return this.attributes.find(
      (definition) => definition.attribute === attribute,
    );
  }

  getAttributeLevel(levels: ModuleLevels, attribute: ModuleAttribute) {
    return levels[attribute] ?? 1;
  }

  getShipModuleAttributeLevel(module: ShipModule, attribute: ModuleAttribute) {
    return this.getAttributeLevel(module.levels, attribute);
  }

  getAttributeValue(levels: ModuleLevels, attribute: ModuleAttribute) {
    const definition = this.getAttribute(attribute);
    if (!definition) return 0;

    return (
      definition.defaultValue *
      (1 +
        Math.max(0, this.getAttributeLevel(levels, attribute) - 1) *
          (definition.upgradeMultiplier ?? DEFAULT_MODULE_UPGRADE_MULTIPLIER))
    );
  }

  getUpgradeCost(
    attribute: ModuleAttribute,
    nextLevel: number,
  ): Partial<Inventory> {
    const definition = this.getAttribute(attribute);
    if (!definition) return {};

    return multiplyInventoryCost(
      definition.materials,
      nextLevel,
      definition.upgradeMultiplier ?? DEFAULT_MODULE_UPGRADE_MULTIPLIER,
    );
  }

  canUpgrade(attribute: ModuleAttribute) {
    return this.getAttribute(attribute) !== undefined;
  }

  upgrade(module: ShipModule, attribute: ModuleAttribute) {
    if (!this.canUpgrade(attribute)) return module;

    const nextModule = {
      ...module,
      levels: {
        ...module.levels,
        [attribute]: this.getShipModuleAttributeLevel(module, attribute) + 1,
      },
    };

    if (attribute !== 'durability') return nextModule;

    const durabilityGain =
      this.getMaxDurability(nextModule.levels) -
      this.getMaxDurability(module.levels);

    return {
      ...nextModule,
      durability: module.durability + durabilityGain,
    };
  }

  getMaxDurability(levels: ModuleLevels) {
    return this.getAttributeValue(levels, 'durability');
  }

  abstract getDurabilityDrainRate(module: ShipModule): ModuleDurabilityDrainRate;

  protected createDefaultLevels(): ModuleLevels {
    return Object.fromEntries(
      this.attributes.map(({ attribute }) => [attribute, 1]),
    ) as ModuleLevels;
  }
}
