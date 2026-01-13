/**
 * User class representing a system user
 */
class User {
    constructor(id, username, email) {
        this.id = id;
        this.username = username;
        this.email = email;
    }

    getDisplayName() {
        return `${this.username} (${this.email})`;
    }
}

/**
 * Authentication service for managing users
 */
class AuthService {
    constructor() {
        this.users = new Map();
    }

    /**
     * Authenticate a user by credentials
     * @param {string} username - The username
     * @param {string} password - The password
     * @returns {Promise<User>} The authenticated user
     */
    async authenticate(username, password) {
        const user = Array.from(this.users.values())
            .find(u => u.username === username);

        if (!user) {
            throw new Error('User not found');
        }

        return user;
    }

    /**
     * Register a new user
     * @param {string} username - The username
     * @param {string} email - The email address
     * @returns {User} The newly created user
     */
    registerUser(username, email) {
        const id = this.users.size + 1;
        const user = new User(id, username, email);
        this.users.set(id, user);
        return user;
    }

    /**
     * Validate email format
     * @param {string} email - Email to validate
     * @returns {boolean} True if valid
     */
    validateEmail(email) {
        return email && email.includes('@');
    }
}

/**
 * Express-like request handler
 */
const handleLogin = async (req, res) => {
    const { username, password } = req.body;
    const authService = new AuthService();

    try {
        const user = await authService.authenticate(username, password);
        res.json({ success: true, user: user.getDisplayName() });
    } catch (error) {
        res.status(401).json({ success: false, error: error.message });
    }
};

// Arrow function example
const createUser = (username, email) => {
    const service = new AuthService();
    return service.registerUser(username, email);
};

// Top-level execution
if (require.main === module) {
    const user = createUser('testuser', 'test@example.com');
    console.log('User created:', user.getDisplayName());
}

module.exports = { User, AuthService, handleLogin, createUser };
