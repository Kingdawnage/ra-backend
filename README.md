# Resume Analyzer Backend

A robust backend service for analyzing resumes using NLP, built with Rust and Axum.

## Features

- 🔐 JWT-based Authentication
- 👥 User Management
- 📄 Resume Upload and Management
- 🤖 NLP-based Resume Analysis
- 🔒 Role-based Access Control
- 🗄️ PostgreSQL Database
- 🐳 Docker Support

## Tech Stack

- **Framework**: Rust with Axum
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT
- **File Handling**: Multipart
- **API Documentation**: OpenAPI (planned)
- **Containerization**: Docker
- **Development**: Cargo

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- Docker (optional)
- Docker Compose (optional)

## Environment Variables

Create a `.env` file in the root directory with the following variables:

```env
# Database (Postgres)
DB_PASS=your_password
DATABASE_URL=postgresql://user:password@host:port/database

# JSON Web Token Credentials
JWT_SECRET=your_secret
JWT_MAXAGE=60
```

## Installation

### Local Development

1. Clone the repository:
```bash
git clone https://github.com/Kingdawnage/ra-backend.git
cd ra-backend
```

2. Install dependencies:
```bash
cargo build
```

3. Run migrations:
```bash
cargo sqlx database setup
```

4. Start the server:
```bash
cargo run
```

### Docker Deployment

1. Build the image:
```bash
docker build -t kingdawnage/ra-backend:1.0.0 .
```

2. Run with Docker Compose:
```bash
docker-compose up
```
# OR build and run in one command:
```bash
docker-compose up --build
```

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register a new user
- `POST /api/auth/login` - Login user
- `POST /api/auth/logout` - Logout user

### Users
- `GET /api/users/me` - Get current user
- `GET /api/users` - Get all users (Admin only)
- `PUT /api/users/:id/name` - Update user name
- `PUT /api/users/:id/role` - Update user role
- `PUT /api/users/:id/password` - Update user password

### Resumes
- `POST /api/resumes/{user_id}/resume` - Upload resume
- `GET /api/resumes/{user_id}/resume/{resume_id}` - Get specific resume
- `DELETE /api/resumes/{user_id}/resume/{resume_id}` - Delete resume
- `GET /api/resumes/{user_id}/resumes` - Get all resumes for user

## Project Structure

```
.
├── src/
│   ├── api/          # API route handlers
│   ├── models/       # Database models
│   ├── routes/       # Route definitions
│   ├── services/     # Business logic
│   └── utils/        # Utility functions
├── migrations/       # Database migrations
├── uploads/         # Temporary file storage
├── Dockerfile       # Docker configuration
├── docker-compose.yml # Docker Compose configuration
└── Cargo.toml       # Rust dependencies
```

## Development

### Running Tests (planned)
```bash
cargo test
```

### Database Migrations
```bash
# Create a new migration
cargo sqlx migrate add <migration_name>

# Run migrations
cargo sqlx migrate run

# Revert migrations
cargo sqlx migrate revert
```

### Development Server
```bash
# Run with cargo-watch for auto-reload
cargo watch -x run
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - SQL toolkit
- [PostgreSQL](https://www.postgresql.org/) - Database 