const API_BASE = "/api";

export class ApiError extends Error {
  public status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export const api = {
  getToken: () => localStorage.getItem("token"),
  
  setToken: (token: string) => localStorage.setItem("token", token),
  
  removeToken: () => localStorage.removeItem("token"),

  request: async (endpoint: string, options: RequestInit = {}) => {
    const token = api.getToken();
    const headers = {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options.headers,
    };

    const res = await fetch(`${API_BASE}${endpoint}`, { ...options, headers });

    if (res.status === 401) {
      api.removeToken();
      window.location.href = "/"; // Force reload/redirect on 401
      throw new ApiError(401, "Unauthorized");
    }

    if (!res.ok) {
      throw new ApiError(res.status, res.statusText);
    }

    // Handle empty responses (like 204 No Content)
    if (res.status === 204) return null;

    // Check if response is JSON
    const contentType = res.headers.get("content-type");
    if (contentType && contentType.includes("application/json")) {
        return res.json();
    }
    
    return res.text();
  }
};
