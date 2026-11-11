// LifeOS Frontend Application
class LifeOSApp {
    constructor() {
        this.isAuthenticated = false;
        this.currentUser = null;
        this.apiBaseUrl = '/api';
        this.state = {
            currentView: 'home',
            userPreferences: {},
            offlineMode: false
        };
        this.init();
    }

    init() {
        this.bindEvents();
        this.checkBackendStatus();
        this.loadUserFromStorage();
    }

    bindEvents() {
        // Auth buttons
        document.getElementById('signin-btn').addEventListener('click', () => this.showAuthModal('signin'));
        document.getElementById('signup-btn').addEventListener('click', () => this.showAuthModal('signup'));
        document.getElementById('get-started').addEventListener('click', () => this.showAuthModal('signup'));

        // Modal controls
        document.getElementById('close-modal').addEventListener('click', () => this.hideAuthModal());
        document.getElementById('switch-auth').addEventListener('click', () => this.toggleAuthMode());
        document.getElementById('auth-form').addEventListener('submit', (e) => this.handleAuthSubmit(e));

        // Close modal when clicking outside
        document.getElementById('auth-modal').addEventListener('click', (e) => {
            if (e.target.id === 'auth-modal') {
                this.hideAuthModal();
            }
        });
    }

    async checkBackendStatus() {
        // Simple backend health check - just log the result
        try {
            const response = await fetch(`${this.apiBaseUrl}/health`);
            if (response.ok) {
                console.log('Backend is online');
            } else {
                console.warn('Backend returned non-OK status:', response.status);
            }
        } catch (error) {
            console.warn('Backend is not reachable:', error);
        }
    }

    loadUserFromStorage() {
        const token = localStorage.getItem('auth_token');
        const userData = localStorage.getItem('user_data');

        if (token && userData) {
            try {
                this.currentUser = JSON.parse(userData);
                this.isAuthenticated = true;
                this.updateUIForAuth();
            } catch (error) {
                console.error('Failed to parse user data:', error);
                this.clearAuthData();
            }
        }
    }

    updateUIForAuth() {
        const unauthenticatedView = document.getElementById('unauthenticated-view');
        const authenticatedView = document.getElementById('authenticated-view');

        if (this.isAuthenticated && this.currentUser) {
            // Show authenticated view
            unauthenticatedView.style.display = 'none';
            authenticatedView.style.display = 'block';

            // Update user email
            document.getElementById('user-email').textContent = this.currentUser.email;

            // Bind logout button
            document.getElementById('logout-btn').addEventListener('click', () => this.logout());

            // Bind canvas action buttons
            document.getElementById('new-note').addEventListener('click', () => this.createNewNote());
            document.getElementById('new-task').addEventListener('click', () => this.createNewTask());
        } else {
            // Show unauthenticated view
            unauthenticatedView.style.display = 'block';
            authenticatedView.style.display = 'none';

            // Re-bind auth buttons
            document.getElementById('signin-btn').addEventListener('click', () => this.showAuthModal('signin'));
            document.getElementById('signup-btn').addEventListener('click', () => this.showAuthModal('signup'));
            document.getElementById('get-started').addEventListener('click', () => this.showAuthModal('signup'));
        }
    }

    showAuthModal(mode = 'signin') {
        const modal = document.getElementById('auth-modal');
        const modalTitle = document.getElementById('modal-title');
        const submitBtn = document.getElementById('auth-submit');
        const switchText = document.getElementById('switch-text');
        const switchBtn = document.getElementById('switch-auth');
        const errorDiv = document.getElementById('auth-error');

        // Clear any previous errors
        errorDiv.textContent = '';

        if (mode === 'signin') {
            modalTitle.textContent = 'Sign In';
            submitBtn.textContent = 'Sign In';
            switchText.textContent = "Don't have an account?";
            switchBtn.textContent = 'Sign Up';
        } else {
            modalTitle.textContent = 'Sign Up';
            submitBtn.textContent = 'Sign Up';
            switchText.textContent = 'Already have an account?';
            switchBtn.textContent = 'Sign In';
        }

        modal.classList.add('show');
        document.getElementById('email').focus();
    }

    hideAuthModal() {
        document.getElementById('auth-modal').classList.remove('show');
        document.getElementById('auth-form').reset();
    }

    toggleAuthMode() {
        const currentTitle = document.getElementById('modal-title').textContent;
        if (currentTitle === 'Sign In') {
            this.showAuthModal('signup');
        } else {
            this.showAuthModal('signin');
        }
    }

    async handleAuthSubmit(e) {
        e.preventDefault();

        const email = document.getElementById('email').value;
        const password = document.getElementById('password').value;
        const isSignUp = document.getElementById('modal-title').textContent === 'Sign Up';
        const errorDiv = document.getElementById('auth-error');
        const submitBtn = document.getElementById('auth-submit');

        // Clear previous errors
        errorDiv.textContent = '';

        // Basic validation
        if (!email || !password) {
            errorDiv.textContent = 'Please fill in all fields';
            return;
        }

        if (password.length < 6) {
            errorDiv.textContent = 'Password must be at least 6 characters';
            return;
        }

        // Disable submit button during request
        submitBtn.disabled = true;
        submitBtn.textContent = 'Processing...';

        try {
            const endpoint = isSignUp ? '/auth/signup' : '/auth/signin';
            const response = await fetch(`${this.apiBaseUrl}${endpoint}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    email,
                    password,
                    ...(isSignUp && { name: email.split('@')[0] })
                })
            });

            const data = await response.json();

            if (response.ok) {
                // Save auth data
                localStorage.setItem('auth_token', data.token);
                localStorage.setItem('refresh_token', data.refresh_token);
                localStorage.setItem('user_data', JSON.stringify(data.user));

                this.currentUser = data.user;
                this.isAuthenticated = true;

                // Update UI
                this.updateUIForAuth();
                this.hideAuthModal();

                // Show success message
                this.showNotification(isSignUp ? 'Account created successfully!' : 'Signed in successfully!', 'success');
            } else {
                errorDiv.textContent = data.error?.message || 'Authentication failed';
            }
        } catch (error) {
            console.error('Auth error:', error);
            errorDiv.textContent = 'Network error. Please try again.';
        } finally {
            submitBtn.disabled = false;
            submitBtn.textContent = isSignUp ? 'Sign Up' : 'Sign In';
        }
    }

    logout() {
        // Simple client-side logout - just clear localStorage
        this.clearAuthData();
        this.updateUIForAuth();
        this.showNotification('Logged out successfully', 'success');
    }

    clearAuthData() {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('user_data');
        this.isAuthenticated = false;
        this.currentUser = null;
    }

    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;

        // Add styles
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 1rem 1.5rem;
            border-radius: var(--radius-md);
            background: ${type === 'success' ? 'var(--success-color)' : 'var(--error-color)'};
            color: white;
            box-shadow: var(--shadow-lg);
            z-index: 1001;
            animation: slideIn 0.3s ease;
        `;

        // Add animation keyframes
        if (!document.querySelector('#notification-styles')) {
            const style = document.createElement('style');
            style.id = 'notification-styles';
            style.textContent = `
                @keyframes slideIn {
                    from { transform: translateX(100%); opacity: 0; }
                    to { transform: translateX(0); opacity: 1; }
                }
                @keyframes slideOut {
                    from { transform: translateX(0); opacity: 1; }
                    to { transform: translateX(100%); opacity: 0; }
                }
            `;
            document.head.appendChild(style);
        }

        document.body.appendChild(notification);

        // Remove after 3 seconds
        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }

    // Canvas methods
    createNewNote() {
        this.showNotification('Creating new note...', 'success');
        // In a real implementation, this would create a note element on the canvas
        const canvas = document.getElementById('workspace-canvas');
        const noteElement = document.createElement('div');
        noteElement.className = 'canvas-note';
        noteElement.innerHTML = `
            <div class="note-header">
                <h3>New Note</h3>
                <button class="note-close">&times;</button>
            </div>
            <div class="note-content" contenteditable="true">
                Start typing your note here...
            </div>
        `;

        // Add close functionality
        noteElement.querySelector('.note-close').addEventListener('click', () => {
            noteElement.remove();
        });

        // Remove placeholder if it exists
        const placeholder = canvas.querySelector('.canvas-placeholder');
        if (placeholder) {
            placeholder.remove();
        }

        canvas.appendChild(noteElement);
    }

    createNewTask() {
        this.showNotification('Creating new task...', 'success');
        // In a real implementation, this would create a task element on the canvas
        const canvas = document.getElementById('workspace-canvas');
        const taskElement = document.createElement('div');
        taskElement.className = 'canvas-task';
        taskElement.innerHTML = `
            <div class="task-header">
                <h3>New Task</h3>
                <button class="task-close">&times;</button>
            </div>
            <div class="task-content">
                <input type="text" class="task-input" placeholder="Enter task description...">
                <div class="task-actions">
                    <button class="btn btn-outline btn-small">Save</button>
                    <button class="btn btn-outline btn-small">Cancel</button>
                </div>
            </div>
        `;

        // Add close functionality
        taskElement.querySelector('.task-close').addEventListener('click', () => {
            taskElement.remove();
        });

        // Remove placeholder if it exists
        const placeholder = canvas.querySelector('.canvas-placeholder');
        if (placeholder) {
            placeholder.remove();
        }

        canvas.appendChild(taskElement);
    }

    // Basic rendering engine methods
    setState(newState) {
        this.state = { ...this.state, ...newState };
        this.render();
    }

    render() {
        // This would be expanded in a real implementation
        // For now, just update the UI based on state
        if (this.state.offlineMode) {
            console.log('App is in offline mode');
        }
    }
}

// Initialize the app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new LifeOSApp();

    // Register service worker for PWA
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register('/sw.js')
            .then(registration => {
                console.log('Service Worker registered with scope:', registration.scope);
            })
            .catch(error => {
                console.log('Service Worker registration failed:', error);
            });
    }
});