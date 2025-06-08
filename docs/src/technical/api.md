# API Reference

_This section documents the HTTP API for Galactic War._

## API Overview

Galactic War provides a REST API for programmatic access to all game functions. The API is designed to support both the web frontend and third-party applications.

### Base URL

```
https://api.galactic-war.com/
```

### Authentication

_Authentication system is planned for future implementation._

## Core Endpoints

### Galaxy Information

#### Get Galaxy Stats

```http
GET /stats
```

Returns overall galaxy statistics including player count, system information, and general metrics.

**Response:**

```json
{
  "players": 42,
  "systems": 200,
  "total_score": 15000,
  "active_events": 150
}
```

#### Get System List

```http
GET /systems
```

Returns a list of all systems in the galaxy with basic information.

### System Operations

#### Get System Details

```http
GET /system/{x}/{y}
```

Get detailed information about a specific system at coordinates (x, y).

**Parameters:**

- `x` - X coordinate of the system
- `y` - Y coordinate of the system

**Response:**

```json
{
  "coordinates": { "x": 50, "y": 75 },
  "resources": {
    "metal": 1250,
    "water": 890,
    "crew": 1100
  },
  "production": {
    "metal": 15,
    "water": 12,
    "crew": 10
  },
  "structures": {
    "colony": 3,
    "asteroidmine": 2,
    "waterharvester": 1
  },
  "events": [
    {
      "type": "construction",
      "completion": 1234567890,
      "structure": "hatchery"
    }
  ]
}
```

#### Build Structure

```http
POST /system/{x}/{y}/build
```

Start construction of a new structure or upgrade an existing one.

**Request Body:**

```json
{
  "structure": "asteroidmine"
}
```

**Response:**

```json
{
  "success": true,
  "event": {
    "completion": 1234567890,
    "type": "construction",
    "structure": "asteroidmine"
  }
}
```

### Structure Information

#### Get Structure Details

```http
GET /system/{x}/{y}/structure/{structure_type}
```

Get detailed information about a specific structure type in a system.

**Parameters:**

- `x`, `y` - System coordinates
- `structure_type` - Type of structure (colony, asteroidmine, etc.)

**Response:**

```json
{
  "level": 2,
  "production": {
    "metal": 8,
    "water": 0,
    "crew": 0
  },
  "next_level_cost": {
    "metal": 15,
    "water": 15,
    "crew": 15,
    "time": 125
  }
}
```

## WebSocket API

Real-time updates are provided via WebSocket connections.

### Connection

```javascript
const ws = new WebSocket("wss://api.galactic-war.com/ws");
```

### Event Types

#### System Updates

Sent when a system's state changes:

```json
{
  "type": "system_update",
  "system": { "x": 50, "y": 75 },
  "data": {
    "resources": { "metal": 1300, "water": 900, "crew": 1100 }
  }
}
```

#### Event Completion

Sent when an event completes:

```json
{
  "type": "event_complete",
  "system": { "x": 50, "y": 75 },
  "event": {
    "type": "construction",
    "structure": "asteroidmine",
    "new_level": 3
  }
}
```

## Error Handling

### Standard Error Response

```json
{
  "error": true,
  "message": "Insufficient resources",
  "code": "INSUFFICIENT_RESOURCES"
}
```

### Common Error Codes

- `INSUFFICIENT_RESOURCES` - Not enough resources for the requested action
- `SYSTEM_NOT_FOUND` - Invalid system coordinates
- `INVALID_STRUCTURE` - Unknown structure type
- `CONSTRUCTION_IN_PROGRESS` - Cannot build while another construction is active

## Rate Limiting

API requests are rate-limited to prevent abuse:

- **Standard Users:** 100 requests per minute
- **Authenticated Users:** 300 requests per minute (planned)
- **Bot Applications:** 1000 requests per minute (planned)

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1234567890
```

## Future API Features

### Planned Endpoints

- Fleet management and movement
- Alliance operations
- User account management
- Real-time battle results
- Statistical data exports

### Advanced Features

- GraphQL API support
- Webhook notifications
- Bulk operations
- Historical data access
- Third-party application registration

_The API is actively being developed. More endpoints and features will be added as the game evolves._
