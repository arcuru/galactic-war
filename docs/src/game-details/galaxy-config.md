# Galaxy Configuration

Galactic War uses YAML configuration files to define galaxy parameters, making it highly customizable and moddable. This configuration-driven approach allows for different game modes, speeds, and mechanics without code changes.

## Configuration Structure

### Galaxy-Level Settings

```yaml
# Galaxy size and population
system_count: 200 # Total number of systems in the galaxy
size:
  x: 100 # Galaxy width in coordinate units
  y: 100 # Galaxy height in coordinate units

# System defaults
systems:
  resources: # Starting resources for new systems
    crew: 200
    metal: 225
    water: 250
  structures: # Available structure types and properties
    # ... structure definitions
```

### Structure Configuration

Each structure type is fully configurable:

```yaml
colony:
  description: "The heart of your empire..."
  starting_level: 1 # Level when system is created
  multiplier: 1.25 # Default scaling multiplier

  production: # Resource production per hour
    metal: 6
    crew: 5
    water: 7
  production_multiplier: 1.2 # Production scaling factor

  cost: # Cost to build/upgrade
    metal: 5
    crew: 5
    water: 5
    time: 1 # Build time in ticks
  cost_multiplier: 1.25 # Cost scaling factor

  storage: # Storage capacity provided
    metal: 1000
    crew: 1000
    water: 1000
  storage_multiplier: 1.1 # Storage scaling factor
```

## Scaling Formulas

All numerical values scale exponentially with structure level:

### Production Scaling

```
Production = Base × Multiplier^(Level-1)
```

For a Colony at level 3 with base metal production of 6 and multiplier 1.2:

```
Level 1: 6 × 1.2^0 = 6 metal/hour
Level 2: 6 × 1.2^1 = 7.2 metal/hour
Level 3: 6 × 1.2^2 = 8.64 metal/hour
```

### Cost Scaling

```
Cost = Base × Multiplier^(Level-1)
```

Building costs increase exponentially, making higher levels much more expensive but also much more powerful.

### Storage Scaling

```
Storage = Base × Multiplier^(Level-1)
```

Storage capacity scales more slowly (typically 1.1x multiplier) to balance resource management.

## Built-in Configurations

### Classic Mode (`classic.yaml`)

Long-term gameplay spanning months:

- **200 systems** in a 100×100 galaxy
- **1.25x cost multiplier** - expensive upgrades
- **1.25x production multiplier** - significant scaling
- **Starting resources:** 200/225/250 Crew/Metal/Water
- **Focus:** Strategic planning and long-term growth

### Blitz Mode (`blitz.yaml`)

Fast-paced gameplay lasting days or weeks:

- **Smaller galaxy** for quicker interaction
- **Faster build times** for rapid expansion
- **Higher starting resources** for immediate action
- **Focus:** Quick decisions and rapid expansion

## Customization Examples

### High-Speed Development Server

```yaml
# Ultra-fast testing configuration
system_count: 50
size: { x: 25, y: 25 }
systems:
  resources: { crew: 1000, metal: 1000, water: 1000 }
  structures:
    colony:
      cost: { time: 1 } # Everything builds in 1 tick
      cost_multiplier: 1.1 # Cheaper upgrades
```

### Resource-Scarce Survival

```yaml
# Challenging resource management
systems:
  resources: { crew: 50, metal: 25, water: 25 }
  structures:
    colony:
      production: { crew: 1, metal: 1, water: 1 }
      production_multiplier: 1.1 # Slow production growth
      storage_multiplier: 1.05 # Limited storage growth
```

### Mega-Galaxy Campaign

```yaml
# Massive long-term world
system_count: 2000
size: { x: 200, y: 200 }
structures:
  colony:
    cost_multiplier: 1.5 # Very expensive upgrades
    production_multiplier: 1.3 # High production scaling
```

## Structure Types Configuration

### Production Structures

Define resource-generating buildings:

```yaml
asteroidmine:
  description: "Extract metal from asteroid fields"
  starting_level: 0 # Must be built
  production:
    metal: 6 # Only produces metal
  cost: { metal: 10, water: 10, crew: 10, time: 100 }
  cost_multiplier: 1.25
  production_multiplier: 1.25
```

### Storage Structures

Define resource storage buildings:

```yaml
storagedepot:
  description: "Increase resource storage capacity"
  starting_level: 0
  storage: { metal: 1000, water: 1000, crew: 1000 }
  cost: { metal: 10, water: 10, crew: 10, time: 100 }
  cost_multiplier: 1.25
  storage_multiplier: 1.25
```

### Multi-Purpose Structures

Combine multiple functions:

```yaml
colony:
  # Produces all resources
  production: { metal: 6, water: 7, crew: 5 }
  # Provides storage for all resources
  storage: { metal: 1000, water: 1000, crew: 1000 }
  # Different multipliers for different functions
  production_multiplier: 1.2
  storage_multiplier: 1.1
  cost_multiplier: 1.25
```

## Configuration Strategy

### Balancing Considerations

**Economic Balance**

- Production vs. cost scaling ratios
- Storage capacity vs. production rates
- Starting resources vs. early costs
- Build times vs. strategic pacing

**Progression Curves**

- Early game accessibility
- Mid-game strategic depth
- Late game exponential scaling
- Meaningful upgrade decisions

**Galaxy Characteristics**

- System density affects expansion opportunities
- Galaxy size impacts travel times (future)
- Starting resources determine early strategy

### Custom Game Modes

**Beginner-Friendly**

- High starting resources
- Low cost multipliers
- Fast build times
- Forgiving storage limits

**Expert Challenge**

- Low starting resources
- High cost multipliers
- Slow build times
- Tight storage constraints

**Experimental**

- Unique structure combinations
- Unusual scaling factors
- Special victory conditions
- Novel mechanics testing

## Future Configuration Options

### Planned Expansions

**Fleet Configuration**

- Ship types and capabilities
- Combat mechanics and balance
- Travel speeds and costs
- Fleet composition limits

**Research Trees**

- Technology prerequisites
- Research costs and times
- Unlockable capabilities
- Tech tree branching

**Victory Conditions**

- Score thresholds
- Territory control
- Special objectives
- Time-based wins

**World Events**

- Random galaxy events
- Resource discoveries
- Natural disasters
- Neutral faction interactions

The configuration system provides the foundation for endless gameplay variety while maintaining balanced and enjoyable experiences.
