# Hackathon Registration Backend

A clean architecture Axum backend for hackathon team registration built with Rust.

## Architecture

This project follows **Clean Architecture** (Bob Uncle's style) with the following layers:

- **Domain**: Core business entities and repository traits
- **Application**: Use cases (business logic)
- **Adapters**: 
- **Persistence**: Repository implementations (SQLx with PostgreSQL)
- **Routes**: HTTP handlers (Axum)

## Prerequisites

- Rust (latest stable version)
- PostgreSQL database
- `.env` file with required environment variables

## Setup

### 1. Environment Variables

Create a `.env` file in the root directory:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/hackathon_db
JWT_SECRET=your-secret-key-here
ACCESS_TOKEN_TTL_SEC=3600
REFRESH_TOKEN_TTL_DAYS=7
```

### 2. Database Setup

1. Create a PostgreSQL database:
```bash
createdb hackathon_db
```

2. The migrations will run automatically when you start the application. The migration file is located at `migrations/001_create_teams_tables.sql`.

Alternatively, you can run migrations manually using sqlx-cli:

```bash
# Install sqlx-cli if you haven't
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

### 3. Build and Run

```bash
# Build the project
cargo build

# Run the server
cargo run
```

The server will start on `http://127.0.0.1:3000`

## Docker Setup

### Quick Start with Docker Compose

The easiest way to run the entire application (including PostgreSQL) is using Docker Compose:

```bash
# Build and start all services (app + database)
docker-compose up --build

# Or run in detached mode
docker-compose up -d --build

# View logs
docker-compose logs -f app

# Stop all services
docker-compose down

# Stop and remove volumes (clears database data)
docker-compose down -v
```

The API will be available at `http://localhost:3000` and PostgreSQL at `localhost:5432`.

### Docker Compose Services

- **postgres**: PostgreSQL 16 database
- **app**: Axum API server

### Environment Variables for Docker

The `docker-compose.yml` file includes default environment variables. To customize them, you can:

1. Edit the `docker-compose.yml` file directly, or
2. Create a `.env` file (Docker Compose will automatically use it)

### Building Docker Image Only

If you want to build just the Docker image without docker-compose:

```bash
# Build the image
docker build -t hackathon-api .

# Run the container (requires external PostgreSQL)
docker run -p 3000:3000 \
  -e DATABASE_URL=postgresql://user:pass@host:5432/db \
  -e JWT_SECRET=your-secret \
  -e ACCESS_TOKEN_TTL_SEC=3600 \
  -e REFRESH_TOKEN_TTL_DAYS=7 \
  hackathon-api
```

### Docker Commands Reference

```bash
# Rebuild only the app (faster if only code changed)
docker-compose build app

# Restart just the app service
docker-compose restart app

# View app logs
docker-compose logs app

# Execute commands in the app container
docker-compose exec app /bin/bash

# Check service status
docker-compose ps
```

## API Endpoints

### Base URL
```
http://localhost:3000/api/v1
```

### 1. Register a Team
**POST** `/teams`

Register a new hackathon team.

**Request Body:**
```json
{
  "team_name": "Team Awesome",
  "idea_description": "Our amazing idea description (max 500 characters)",
  "impact_description": "The impact our idea will make (max 500 characters)",
  "members": [
    {
      "name": "John Doe",
      "tms_id": "TMS123",
      "email": "john@example.com",
      "is_team_lead": true
    },
    {
      "name": "Jane Smith",
      "tms_id": "TMS456",
      "email": "jane@example.com",
      "is_team_lead": false
    }
  ]
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid-here",
  "team_name": "Team Awesome",
  "idea_description": "...",
  "impact_description": "...",
  "members": [...],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

**Validation Rules:**
- Team name must be unique
- Each member's email and TMS ID must be unique across all teams
- Team must have exactly one team lead
- Idea and impact descriptions must be max 500 characters each

### 2. List Teams (with Pagination, Filter, and Search)
**GET** `/teams`

List all teams with pagination, filtering, and search capabilities.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 10, max: 100)
- `search` (optional): Search in team name, idea description, or impact description
- `team_name` (optional): Filter by team name (partial match)

**Example:**
```
GET /teams?page=1&page_size=10&search=awesome&team_name=Team
```

**Response:** `200 OK`
```json
{
  "teams": [...],
  "total": 50,
  "page": 1,
  "page_size": 10,
  "total_pages": 5
}
```

### 3. Get Team by ID
**GET** `/teams/:id`

Get a specific team by its ID.

**Response:** `200 OK`
```json
{
  "id": "uuid-here",
  "team_name": "Team Awesome",
  ...
}
```

### 4. Check if Member is Registered
**GET** `/teams/check-member`

Check if a user (by email or TMS ID) is already registered in a team.

**Query Parameters:**
- `email` (optional): Member's email
- `tms_id` (optional): Member's TMS ID

**Note:** At least one of `email` or `tms_id` must be provided.

**Example:**
```
GET /teams/check-member?email=john@example.com
```

**Response:** `200 OK`
```json
{
  "is_registered": true,
  "team_name": "Team Awesome",
  "team_id": "uuid-here"
}
```

### 5. Get Total Team Count
**GET** `/teams/count`

Get the total number of registered teams.

**Response:** `200 OK`
```json
{
  "total_teams": 42
}
```

### 6. Health Check
**GET** `/health`

Check if the API is running.

**Response:** `200 OK`
```json
{
  "status": "ok",
  "database": "connected"
}
```

## Database Schema

### Teams Table
- `id` (UUID, Primary Key)
- `team_name` (VARCHAR(255), Unique)
- `idea_description` (TEXT)
- `impact_description` (TEXT)
- `created_at` (TIMESTAMPTZ)
- `updated_at` (TIMESTAMPTZ)

### Team Members Table
- `id` (UUID, Primary Key)
- `team_id` (UUID, Foreign Key to teams)
- `name` (VARCHAR(255))
- `tms_id` (VARCHAR(255), Unique)
- `email` (VARCHAR(255), Unique)
- `is_team_lead` (BOOLEAN)
- `created_at` (TIMESTAMPTZ)
- `updated_at` (TIMESTAMPTZ)

## Error Handling

The API returns appropriate HTTP status codes:

- `200 OK`: Success
- `201 Created`: Resource created successfully
- `400 Bad Request`: Validation error
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., duplicate team name or member)
- `500 Internal Server Error`: Server error

Error responses include a message describing the issue.

## Testing the API

You can use `curl` or any HTTP client like Postman or Insomnia:

```bash
# Register a team
curl -X POST http://localhost:3000/api/v1/teams \
  -H "Content-Type: application/json" \
  -d '{
    "team_name": "Team Awesome",
    "idea_description": "Our idea",
    "impact_description": "Our impact",
    "members": [
      {
        "name": "John Doe",
        "tms_id": "TMS123",
        "email": "john@example.com",
        "is_team_lead": true
      }
    ]
  }'

# List teams
curl http://localhost:3000/api/v1/teams?page=1&page_size=10

# Check member
curl http://localhost:3000/api/v1/teams/check-member?email=john@example.com

# Get team count
curl http://localhost:3000/api/v1/teams/count
```

## Project Structure

```
src/
├── domain/           # Domain layer (entities, repository traits)
│   ├── team.rs
│   └── repository.rs
├── application/      # Application layer (use cases)
│   └── team_usecase.rs
├── adapters/         # Adapters layer
│   ├── persistence/  # Repository implementations
│   │   └── team_repository.rs
│   └── routes/       # HTTP handlers
│       └── teams.rs
└── infra/            # Infrastructure (config, DB setup)
    ├── db.rs
    └── setup.rs
```

## Learning Resources

Since you're learning Rust and Axum, here are some key concepts used in this project:

1. **Async/Await**: All database operations are async
2. **Traits**: Repository pattern using traits for abstraction
3. **Error Handling**: Custom error types with `Result<T, E>`
4. **SQLx**: Type-safe SQL queries with compile-time checking
5. **Clean Architecture**: Separation of concerns across layers

## Notes

- Migrations run automatically on application startup
- All timestamps are in UTC
- Team names and member emails/TMS IDs are case-sensitive for uniqueness
- The API uses JSON for request/response bodies
- CORS is configured for local development (ports 3000 and 5173)

