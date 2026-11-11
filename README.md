# LifeOS

A web-based personal operating system for managing your digital life.

## Overview

LifeOS is a Progressive Web Application (PWA) designed to work offline-first, with a Rust-powered backend for synchronization and user management. The application prioritizes minimal dependencies, performance, and reliability.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (PWA)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Vanilla JS  │  │    HTML     │  │        CSS          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Custom Rendering Engine                     ││
│  │  • State Management  • Reactive Updates  • DOM Diffing  ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Browser Storage Layer                       ││
│  │  • LocalStorage (settings/preferences)                   ││
│  │  • IndexedDB (structured data/documents)                 ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/REST API
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Rust)                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │Static Files │  │    Auth     │  │   Token Service     │  │
│  │   Server    │  │   Module    │  │   (JWT/Sessions)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```
lifeos/
├── frontend/               # Progressive Web App
│   └── README.md           # Frontend-specific documentation
│
├── backend/                # Rust API server
│   └── README.md           # Backend-specific documentation
│
├── docs/                   # Additional documentation
│   ├── architecture.md
│   ├── api.md
│   └── deployment.md
│
└── README.md               # This file
```

---

## Non-Functional Requirements

### Performance
- First Contentful Paint (FCP) < 1.5s
- Time to Interactive (TTI) < 3s
- Lighthouse Performance Score > 90

### Security
- HTTPS only in production
- Secure cookie attributes (HttpOnly, Secure, SameSite)
- CORS configuration
- Input sanitization
- SQL injection prevention (parameterized queries)

### Reliability
- Graceful degradation when offline
- Data conflict resolution for sync
- Automatic retry with exponential backoff

### Maintainability
- Comprehensive documentation
- Unit and integration tests
- CI/CD pipeline

---

## Development Phases

### Phase 1: Foundation
- [ ] Set up project structure
- [ ] Implement basic rendering engine (state + render)
- [ ] Create Rust backend skeleton
- [ ] Static file serving

### Phase 2: Core Features
- [ ] Complete component system
- [ ] IndexedDB integration
- [ ] User authentication (signup/signin)
- [ ] Token management

### Phase 3: PWA & Offline
- [ ] Service worker implementation
- [ ] Offline data persistence
- [ ] Background sync
- [ ] App manifest and installability

### Phase 4: Polish & Deploy
- [ ] Performance optimization
- [ ] Security hardening
- [ ] Documentation
- [ ] Deployment setup

---

## Getting Started

See individual README files for setup instructions:

- [Frontend Documentation](./frontend/README.md)
- [Backend Documentation](./backend/README.md)

---

## License

TBD

---

## Contributing

TBD
