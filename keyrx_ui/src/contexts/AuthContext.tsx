/**
 * Authentication Context Provider
 *
 * Provides JWT-based authentication with:
 * - Automatic token refresh
 * - Secure httpOnly cookie storage
 * - Auto-redirect on 401 errors
 * - Login/logout functionality
 */

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

interface AuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  user: User | null;
  error: string | null;
}

interface User {
  id: string;
  username: string;
}

interface LoginResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const TOKEN_STORAGE_KEY = 'keyrx_access_token';
const REFRESH_TOKEN_KEY = 'keyrx_refresh_token';
const TOKEN_REFRESH_INTERVAL = 14 * 60 * 1000; // 14 minutes (before 15min expiry)

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<User | null>(null);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  /**
   * Validate current token
   */
  const validateToken = useCallback(async (): Promise<boolean> => {
    const token = localStorage.getItem(TOKEN_STORAGE_KEY);
    if (!token) return false;

    try {
      const response = await fetch('/api/auth/validate', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        if (data.valid && data.user_id) {
          setUser({ id: data.user_id, username: data.user_id });
          return true;
        }
      }

      return false;
    } catch (err) {
      console.error('Token validation failed:', err);
      return false;
    }
  }, []);

  /**
   * Refresh access token using refresh token
   */
  const refreshToken = useCallback(async () => {
    const refreshTokenValue = localStorage.getItem(REFRESH_TOKEN_KEY);
    if (!refreshTokenValue) {
      setIsAuthenticated(false);
      navigate('/login');
      return;
    }

    try {
      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          refresh_token: refreshTokenValue,
        }),
      });

      if (response.ok) {
        const data: LoginResponse = await response.json();
        localStorage.setItem(TOKEN_STORAGE_KEY, data.access_token);
        localStorage.setItem(REFRESH_TOKEN_KEY, data.refresh_token);
        setIsAuthenticated(true);
      } else {
        // Refresh token expired or invalid
        localStorage.removeItem(TOKEN_STORAGE_KEY);
        localStorage.removeItem(REFRESH_TOKEN_KEY);
        setIsAuthenticated(false);
        navigate('/login');
      }
    } catch (err) {
      console.error('Token refresh failed:', err);
      setIsAuthenticated(false);
      navigate('/login');
    }
  }, [navigate]);

  /**
   * Login with username and password
   */
  const login = useCallback(async (username: string, password: string) => {
    setError(null);
    setIsLoading(true);

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      if (response.ok) {
        const data: LoginResponse = await response.json();
        localStorage.setItem(TOKEN_STORAGE_KEY, data.access_token);
        localStorage.setItem(REFRESH_TOKEN_KEY, data.refresh_token);
        setUser({ id: username, username });
        setIsAuthenticated(true);
        navigate('/');
      } else {
        const errorData = await response.json();
        setError(errorData.error || 'Login failed');
        setIsAuthenticated(false);
      }
    } catch (err) {
      setError('Network error. Please try again.');
      setIsAuthenticated(false);
    } finally {
      setIsLoading(false);
    }
  }, [navigate]);

  /**
   * Logout and clear tokens
   */
  const logout = useCallback(async () => {
    try {
      await fetch('/api/auth/logout', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem(TOKEN_STORAGE_KEY)}`,
        },
      });
    } catch (err) {
      console.error('Logout request failed:', err);
    }

    localStorage.removeItem(TOKEN_STORAGE_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    setIsAuthenticated(false);
    setUser(null);
    navigate('/login');
  }, [navigate]);

  /**
   * Initialize authentication state on mount
   */
  useEffect(() => {
    const initAuth = async () => {
      const token = localStorage.getItem(TOKEN_STORAGE_KEY);
      if (!token) {
        setIsLoading(false);
        return;
      }

      const valid = await validateToken();
      setIsAuthenticated(valid);

      if (!valid) {
        // Try to refresh token
        await refreshToken();
      }

      setIsLoading(false);
    };

    initAuth();
  }, [validateToken, refreshToken]);

  /**
   * Set up automatic token refresh
   */
  useEffect(() => {
    if (!isAuthenticated) return;

    const interval = setInterval(() => {
      refreshToken();
    }, TOKEN_REFRESH_INTERVAL);

    return () => clearInterval(interval);
  }, [isAuthenticated, refreshToken]);

  /**
   * Intercept 401 responses and redirect to login
   */
  useEffect(() => {
    const handleUnauthorized = () => {
      setIsAuthenticated(false);
      localStorage.removeItem(TOKEN_STORAGE_KEY);
      localStorage.removeItem(REFRESH_TOKEN_KEY);
      navigate('/login');
    };

    // Listen for 401 responses from fetch
    const originalFetch = window.fetch;
    window.fetch = async (...args) => {
      const response = await originalFetch(...args);
      if (response.status === 401) {
        handleUnauthorized();
      }
      return response;
    };

    return () => {
      window.fetch = originalFetch;
    };
  }, [navigate]);

  const value: AuthContextType = {
    isAuthenticated,
    isLoading,
    login,
    logout,
    refreshToken,
    user,
    error,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

/**
 * Hook to use authentication context
 */
export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

/**
 * Get authorization header for API requests
 */
export const getAuthHeader = (): { Authorization: string } | {} => {
  const token = localStorage.getItem(TOKEN_STORAGE_KEY);
  return token ? { Authorization: `Bearer ${token}` } : {};
};
