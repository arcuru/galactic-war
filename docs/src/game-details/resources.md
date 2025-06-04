# Resources

Galactic War features three core resources that drive all economic activity in your empire. Each resource serves specific purposes and is produced by different structures.

## Resource Types

### 🔩 Metal
**Primary Construction Material**

Metal is the backbone of your infrastructure, required for:
- Building and upgrading most structures
- Constructing ships and fleets (planned)
- Advanced technologies and upgrades

**Production Sources:**
- **Colony** - Provides base metal production
- **Asteroid Mine** - Primary metal production facility

### 💧 Water
**Life Support Resource**

Water sustains your empire and enables expansion:
- Required for most construction projects
- Essential for crew survival and growth
- Used in advanced manufacturing processes

**Production Sources:**
- **Colony** - Provides base water production  
- **Water Harvester** - Specialized water collection from comets and ice

### 👥 Crew
**Human Capital**

Crew represents your population and workforce:
- Required for operating structures and systems
- Needed for constructing and upgrading buildings
- Essential for manning ships and conducting operations

**Production Sources:**
- **Colony** - Natural population growth
- **Hatchery** - Accelerated crew production and training

## Resource Mechanics

### Production
- All resources are produced continuously over time
- Production rates are expressed as "per hour" (3600 ticks)
- Multiple structures of the same type stack their production
- Higher level structures produce exponentially more resources

### Storage
- Each system has limited storage capacity for each resource
- **Storage Depot** structures increase storage capacity
- Resources are lost if they exceed storage limits
- Plan storage upgrades alongside production increases

### Consumption
All construction and upgrades consume resources instantly when started. Make sure you have sufficient resources before beginning projects.

## Resource Management Strategy

### Early Game
1. **Balance Production** - Ensure steady growth in all three resources
2. **Upgrade Colony First** - Your colony provides base production of all resources
3. **Don't Neglect Storage** - Build Storage Depots to prevent resource waste

### Mid Game
1. **Specialize Systems** - Focus different systems on different resource types
2. **Plan Ahead** - High-level structures require substantial resource investments
3. **Monitor Ratios** - Maintain appropriate resource ratios for your expansion plans

### Resource Formulas

Production and costs scale exponentially with structure levels:

```
Production = Base × Multiplier^(Level-1)
Cost = Base × Multiplier^(Level-1)
```

Where:
- **Base** = Level 1 production/cost values
- **Multiplier** = Scaling factor (typically 1.25)
- **Level** = Current structure level

This exponential scaling means higher-level structures are dramatically more powerful but also much more expensive. 