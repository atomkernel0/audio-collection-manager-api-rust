# Audio Collection Manager API (Rust)

# README WIP

A high-performance audio collection management API built with Rust, Axum, and SurrealDB.

## Features

- 🎵 Song, Album, and Artist management
- 👤 User authentication and authorization
- ⭐ Favorites and playlists
- 🔍 Advanced search capabilities
- 📊 Listen tracking and statistics
- 🏆 Badge system for user achievements
- 🛡️ Rate limiting for authenticated and anonymous users

## Rate Limiting

The API implements two types of rate limiting:

### Authenticated Users
- Maximum 100 listens per hour
- Maximum 10 listens per minute
- Cannot listen to the same song twice within 70% of its duration (minimum 10 seconds)

### Anonymous Users (IP-based)
- Maximum 5 listens per minute per IP address
- Encourages users to sign in for unlimited listening
- Automatic cleanup of old tracking records

## Database Setup

Run the following migration scripts in order:

```bash
# Main database schema
surreal import --conn http://localhost:8000 --user root --pass root --ns your_namespace --db your_database database_schema.surql

# Anonymous listen tracking (for IP-based rate limiting)
surreal import --conn http://localhost:8000 --user root --pass root --ns your_namespace --db your_database database_anonymous_listen_log.surql

# Other migrations as needed
surreal import --conn http://localhost:8000 --user root --pass root --ns your_namespace --db your_database database_events_migration.surql
```

## Environment Variables

Create a `.env` file based on `.env.example`:

```env
DB_URL=http://localhost:8000
DB_NS=your_namespace
DB_NAME=your_database
DB_USER=root
DB_PASSWORD=root
JWT_SECRET=your_secret_key_here
JWT_EXPIRATION=86400
BIND_HOST=0.0.0.0
PORT=8080
```

## Running the API

```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/audio-collection-manager-rust
```

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user

### Songs
- `POST /api/song/{song_id}/listen` - Record a song listen (supports both authenticated and anonymous users)
- `GET /api/song/recents` - Get user's recent listens (requires auth)
- `GET /api/song/{song_id}/album` - Get album from song

### Albums
- `GET /api/albums` - List all albums
- `GET /api/albums/{album_id}` - Get album details

### Artists
- `GET /api/artists` - List all artists
- `GET /api/artists/{artist_id}` - Get artist details

### Search
- `GET /api/search?q={query}` - Search across songs, albums, and artists

### User (Protected)
- `GET /api/user/profile` - Get user profile
- `GET /api/user/top-songs` - Get user's top songs
- `GET /api/user/badges` - Get user badges

### Playlists (Protected)
- `GET /api/playlist` - List user playlists
- `POST /api/playlist` - Create playlist
- `GET /api/playlist/{playlist_id}` - Get playlist details

### Favorites (Protected)
- `POST /api/favorites/song/{song_id}` - Favorite a song
- `DELETE /api/favorites/song/{song_id}` - Unfavorite a song

## Architecture

- **Framework**: Axum (async web framework)
- **Database**: SurrealDB (multi-model database)
- **Authentication**: JWT tokens
- **Rate Limiting**: In-memory cache + database tracking
- **Logging**: tracing + tracing-subscriber

## Security Features

- JWT-based authentication
- Password hashing with bcrypt
- IP-based rate limiting for anonymous users
- User-based rate limiting for authenticated users
- CORS protection
- Request tracing and logging

## License

See LICENSE file for details.
