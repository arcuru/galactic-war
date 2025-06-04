# Events and Time System

Galactic War operates on a sophisticated event-driven system that manages all game activities in real-time. Understanding how events work is crucial for effective gameplay and strategic planning.

## Time System Overview

### Ticks
The fundamental unit of time in Galactic War is the **tick**:
- 1 tick = 1 second of real time (configurable per galaxy)
- All game calculations happen in discrete tick intervals
- Events are scheduled and processed at specific tick times
- Production rates are calculated per hour (3600 ticks)

### Real-Time Gameplay
Unlike turn-based games, Galactic War runs continuously:
- Resource production happens every tick
- Construction projects complete automatically
- Events process without player intervention
- Multiple activities can occur simultaneously

## Event System

### Event Types

**Production Events**
- Metal, Water, and Crew generation
- Triggered automatically every hour (3600 ticks)
- Creates the next production event when completed
- Provides continuous resource income

**Construction Events** 
- Building and upgrading structures
- Duration varies by structure type and level
- Consumes resources when started
- Activates structure when completed

**Future: Fleet Events**
- Ship movement between systems
- Fleet arrivals and departures
- Combat resolution
- Colonization attempts

### Event Processing

**Automatic Processing**
Events are processed automatically when their completion time arrives:
1. Event completion time is checked each tick
2. Completed events trigger their effects
3. New events may be created as consequences
4. Game state is updated immediately

**Event Ordering**
- Events are processed in the order they complete
- Multiple events completing on the same tick are processed sequentially
- Event consequences may create additional events
- No player intervention required

### Event Information

Players can view scheduled events for their systems:
- **Event Type** - What will happen (production, construction, etc.)
- **Completion Time** - When the event will complete
- **Associated Structure** - Which structure is involved (if any)
- **Expected Outcome** - What will happen when the event completes

## Resource Production Mechanics

### Production Cycles
Resource production operates on continuous cycles:

1. **Initial Production Event** created when system is established
2. **Event Completes** after 3600 ticks (1 hour)
3. **Resources Added** to system storage
4. **Next Event Scheduled** for the following hour
5. **Cycle Repeats** indefinitely

### Production Calculation
When a production event completes:
- All production structures are evaluated
- Individual structure production is calculated based on level
- Total production is summed across all structures
- Resources are added to system storage (up to capacity limits)

### Storage Limits
Resource production respects storage capacity:
- Production that exceeds storage capacity is lost
- Players must manage storage expansion
- Storage Depots increase capacity
- Warning systems alert to approaching limits

## Construction Mechanics

### Construction Process
Building and upgrading follows a specific timeline:

1. **Resource Check** - Verify sufficient resources available
2. **Resource Consumption** - Resources deducted immediately
3. **Construction Event Scheduled** - Event created with completion time
4. **Waiting Period** - Construction time elapses
5. **Completion** - Structure becomes active automatically

### Construction Time Calculation
Construction duration depends on:
- **Structure Type** - Different base construction times
- **Level** - Higher levels take longer (exponential scaling)
- **Galaxy Configuration** - Server-wide time multipliers

Formula: `Time = Base_Time × Multiplier^(Level-1)`

### Multiple Construction Projects
- Each system can have multiple construction projects simultaneously
- Projects complete independently
- No construction queues (yet) - each project must be started manually
- Resource planning becomes crucial for multiple projects

## Strategic Implications

### Timing Considerations

**Resource Management**
- Plan production cycles around construction needs
- Ensure storage capacity before major production upgrades
- Time resource gathering with planned construction projects

**Construction Scheduling**
- Consider construction times when planning expansion
- Higher-level structures require significant time investment
- Balance immediate needs vs. long-term benefits

**Event Coordination**
- Multiple events completing simultaneously can overwhelm storage
- Stagger construction projects to maintain resource flow
- Plan ahead for periods of high resource consumption

### Future Event System Expansions

**Fleet Travel Time**
- Fleets will take time to travel between systems
- Distance affects travel duration
- Strategic positioning becomes important

**Research Projects**
- Technology research will use the event system
- Long-term research projects spanning days or weeks
- Research completion unlocks new capabilities

**Diplomatic Events**
- Alliance proposals and responses
- Treaty negotiations with time limits
- Coordinated alliance actions

**Combat Events**
- Fleet arrival notifications
- Battle resolution timing
- Defensive preparation windows

The event system provides the foundation for all these future features while maintaining the current smooth gameplay experience. 