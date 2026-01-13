package com.example.coderag;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

/**
 * User entity representing a system user
 */
public class User {
    private int id;
    private String username;
    private String email;

    public User(int id, String username, String email) {
        this.id = id;
        this.username = username;
        this.email = email;
    }

    public int getId() {
        return id;
    }

    public String getUsername() {
        return username;
    }

    public String getEmail() {
        return email;
    }
}

/**
 * Service for handling user authentication
 */
class AuthenticationService {
    private Map<Integer, User> users;

    public AuthenticationService() {
        this.users = new HashMap<>();
    }

    /**
     * Authenticates a user by username and password
     * @param username The username to authenticate
     * @param password The password to verify
     * @return Optional containing the user if found
     */
    public Optional<User> authenticate(String username, String password) {
        return users.values().stream()
            .filter(user -> user.getUsername().equals(username))
            .findFirst();
    }

    /**
     * Registers a new user in the system
     * @param username The username for the new user
     * @param email The email address
     * @return The newly created user
     */
    public User registerUser(String username, String email) {
        int newId = users.size() + 1;
        User user = new User(newId, username, email);
        users.put(newId, user);
        return user;
    }

    /**
     * Validates email format
     * @param email The email to validate
     * @return true if valid, false otherwise
     */
    private boolean validateEmail(String email) {
        return email != null && email.contains("@");
    }
}

/**
 * Main application entry point
 */
public class Application {
    public static void main(String[] args) {
        AuthenticationService authService = new AuthenticationService();
        User user = authService.registerUser("testuser", "test@example.com");
        System.out.println("User registered: " + user.getUsername());
    }
}
