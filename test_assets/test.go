package main

import (
	"fmt"
	"net/http"
)

// User represents a user in the system
type User struct {
	ID       int
	Username string
	Email    string
}

// AuthService handles authentication
type AuthService struct {
	users map[int]User
}

// NewAuthService creates a new authentication service
func NewAuthService() *AuthService {
	return &AuthService{
		users: make(map[int]User),
	}
}

// Authenticate verifies user credentials
func (s *AuthService) Authenticate(username, password string) (*User, error) {
	// Simplified authentication logic
	for _, user := range s.users {
		if user.Username == username {
			return &user, nil
		}
	}
	return nil, fmt.Errorf("user not found")
}

// RegisterUser adds a new user to the system
func (s *AuthService) RegisterUser(username, email string) (*User, error) {
	user := User{
		ID:       len(s.users) + 1,
		Username: username,
		Email:    email,
	}
	s.users[user.ID] = user
	return &user, nil
}

// HandleLogin processes HTTP login requests
func HandleLogin(w http.ResponseWriter, r *http.Request) {
	username := r.FormValue("username")
	password := r.FormValue("password")
	
	service := NewAuthService()
	user, err := service.Authenticate(username, password)
	
	if err != nil {
		http.Error(w, "Authentication failed", http.StatusUnauthorized)
		return
	}
	
	fmt.Fprintf(w, "Welcome, %s!", user.Username)
}

func main() {
	http.HandleFunc("/login", HandleLogin)
	fmt.Println("Server starting on :8080")
	http.ListenAndServe(":8080", nil)
}
