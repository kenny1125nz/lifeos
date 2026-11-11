# LifeOS Frontend

A Progressive Web App (PWA) built with vanilla JavaScript, HTML, and CSS.

## Overview

The LifeOS frontend is a modern web application designed to work offline-first with minimal dependencies. It features a custom rendering engine for state management and reactive updates.

## Features

- **Progressive Web App (PWA)**: Installable, works offline, feels like a native app
- **Vanilla JavaScript**: No frameworks for maximum performance and control
- **Custom Rendering Engine**: Home-built reactive rendering system
- **Offline-First**: Service worker caches assets and provides offline functionality
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Modern CSS**: CSS Grid, Flexbox, Custom Properties

## Project Structure

```
frontend/
├── public/
│   ├── index.html          # Main HTML file
│   ├── css/
│   │   └── style.css       # Main stylesheet
│   ├── js/
│   │   └── app.js          # Main application logic
│   ├── icons/              # PWA icons (placeholder)
│   ├── manifest.json       # PWA manifest
│   └── sw.js              # Service worker
└── README.md
```

## Key Components

### 1. Application Architecture

The frontend follows a modular architecture with these main components:

#### `LifeOSApp` Class
- Main application controller
- Handles authentication, state management, and UI updates
- Manages API communication with the backend

#### Custom Rendering Engine
- Simple state management system
- Reactive updates based on state changes
- DOM manipulation optimized for performance

### 2. State Management

```javascript
state = {
    currentView: 'home',
    userPreferences: {},
    offlineMode: false
};

setState(newState) {
    this.state = { ...this.state, ...newState };
    this.render();
}
```

### 3. Authentication Flow

1. User clicks Sign In/Sign Up
2. Modal appears with form
3. API call to backend (`/api/auth/signin` or `/api/auth/signup`)
4. Tokens stored in localStorage
5. UI updates to show authenticated state

### 4. Service Worker

- Caches static assets for offline use
- Implements cache-first strategy for non-API requests
- Supports background sync (placeholder)
- Handles push notifications

## Development

### Prerequisites

- Modern web browser (Chrome, Firefox, Safari, Edge)
- Local web server for development

### Running Locally

1. Start the Rust backend server
2. Serve the frontend files (the backend serves them at `/`)
3. Open `http://localhost:3000` in your browser

### Building for Production

The frontend is ready to serve as-is. For production:

1. Ensure all static assets are properly cached by the service worker
2. Update the `CACHE_NAME` in `sw.js` for cache busting
3. Generate proper PWA icons in multiple sizes
4. Configure HTTPS for PWA installation

## PWA Features

### Installation

Users can install LifeOS as a standalone app:

1. Visit the site in a supported browser (Chrome, Edge, Safari)
2. Look for the install prompt or use the browser menu
3. The app will be added to the home screen/app launcher

### Offline Support

- Static assets cached on first visit
- App works without internet connection
- API requests fail gracefully when offline
- Background sync for pending operations

### Push Notifications

- Service worker handles push events
- Shows notifications even when app is closed
- Clicking notifications opens the app

## Browser Support

- Chrome 54+ (full PWA support)
- Firefox 63+ (full PWA support)
- Safari 11.1+ (partial PWA support)
- Edge 79+ (full PWA support)

## Performance

- First Contentful Paint (FCP) < 1.5s target
- Time to Interactive (TTI) < 3s target
- Lighthouse Performance Score > 90 target
- Minimal JavaScript bundle size (vanilla JS)

## Future Enhancements

1. **Advanced Rendering Engine**
   - Virtual DOM implementation
   - Component system with lifecycle hooks
   - Scoped CSS support

2. **Offline Data Management**
   - IndexedDB integration for structured data
   - Conflict resolution for sync
   - Background sync with retry logic

3. **Advanced Features**
   - Drag-and-drop interface
   - Keyboard shortcuts
   - Theme system
   - Plugin architecture

## License

TBD

## Contributing

TBD
