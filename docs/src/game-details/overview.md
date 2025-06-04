# Game Overview

Galactic War is a real-time strategy game set in a vast galaxy filled with solar systems waiting to be claimed and developed.

## Core Concept

You start as the ruler of a single **Colony** in a **Solar System**. Your goal is to build an empire spanning multiple systems throughout the galaxy by:

1. **Producing Resources** - Build structures that generate Metal, Water, and Crew
2. **Expanding Infrastructure** - Upgrade existing structures and build new ones
3. **Strategic Growth** - Plan your development over time as everything happens in real-time
4. **Future: Conquest** - Use fleets to explore, raid, and colonize other systems

## Galaxy Structure

### Coordinates
Each system in the galaxy has coordinates in the form `(X, Y)`. The galaxy is a 2D grid where you can view nearby systems and plan expansion routes.

### System Types
Currently, all systems are habitable colonies, but future versions will include:
- **Habitable Systems** - Full colonies with all building options
- **Outposts** - Limited building capabilities, require supply lines
- **Resource-Specific Systems** - Specialized for certain resource types

## Time System

The game operates in **ticks**, where:
- 1 tick = 1 second of real time (configurable)
- All production rates are given "per hour" (3600 ticks)
- Buildings take time to construct (measured in ticks)
- Events are scheduled and processed automatically

## Victory and Scoring

Your **Score** is calculated by summing all your structure levels across all systems. Growing a structure from level X to level X+1 increases your score by X+1 points.

For example:
- A level 3 Colony is worth 1+2+3 = 6 points
- Upgrading it to level 4 adds 4 more points (total: 10 points)

## Game Modes

Different galaxy configurations provide varied gameplay experiences:

### Classic Mode
- Slow-paced, long-term gameplay spanning months
- Large galaxy with many systems
- Emphasis on careful planning and resource management

### Blitz Mode  
- Fast-paced games lasting days or weeks
- Smaller galaxy with accelerated timers
- Quick decision-making and rapid expansion

All game parameters are configurable, allowing for custom experiences tailored to different play styles and time commitments. 