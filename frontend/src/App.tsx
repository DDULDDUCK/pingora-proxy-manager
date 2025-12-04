import { useState } from "react";
import { Toaster, toast } from "sonner";
import { 
  Lock, LogOut, Loader2
} from "lucide-react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from "@/components/ui/card";

import { DashboardTab } from "@/components/dashboard/DashboardTab";
import { HostsTab } from "@/components/hosts/HostsTab";
import { StreamsTab } from "@/components/streams/StreamsTab";
import { LogsTab } from "@/components/dashboard/LogsTab";
import { SettingsTab } from "@/components/dashboard/SettingsTab";
import { AccessListsTab } from "@/components/access/AccessListsTab"; // Added
import { api } from "@/lib/api";
import ppnIcon from '@/assets/ppnicon.png'; 

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  const [token, setToken] = useState<string | null>(api.getToken());
  
  // Login State
  const [loginId, setLoginId] = useState("");
  const [loginPw, setLoginPw] = useState("");
  const [loginLoading, setLoginLoading] = useState(false);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoginLoading(true);
    try {
      const res = await fetch("/api/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: loginId, password: loginPw }),
      });
      
      if (res.ok) {
        const data = await res.json();
        api.setToken(data.token);
        setToken(data.token);
        toast.success(`Welcome back, ${loginId}!`);
      } else {
        toast.error("Invalid credentials.");
      }
    } catch (err) {
      toast.error("Login server error.");
    } finally {
      setLoginLoading(false);
    }
  };

  const handleLogout = () => {
    api.removeToken();
    setToken(null);
    toast.info("Logged out successfully");
  };

  if (!token) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-100 p-4">
        <Toaster position="top-center" />
        <Card className="w-full max-w-sm shadow-lg">
          <CardHeader className="text-center">
            <div className="mx-auto bg-primary/10 p-3 rounded-full w-fit mb-2">
              <Lock className="h-6 w-6 text-primary" />
            </div>
            <CardTitle className="text-xl">Pingora Manager</CardTitle>
            <CardDescription>Sign in to manage your proxy</CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="username">Username</Label>
                <Input 
                  id="username" 
                  value={loginId} 
                  onChange={(e) => setLoginId(e.target.value)} 
                  disabled={loginLoading}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input 
                  id="password" 
                  type="password" 
                  value={loginPw} 
                  onChange={(e) => setLoginPw(e.target.value)} 
                  disabled={loginLoading}
                />
              </div>
              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Sign In"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <QueryClientProvider client={queryClient}>
      <MainLayout onLogout={handleLogout} />
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}

// --- Main Layout Component ---

function MainLayout({ onLogout }: { onLogout: () => void }) {
  return (
    <div className="min-h-screen bg-slate-50/50 p-6 md:p-10">
      <Toaster position="top-right" />
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg">
              <img src={ppnIcon} alt="App Icon" className="h-8 w-8" />
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight text-slate-900">
                Pingora Proxy Manager
              </h1>
              <p className="text-sm text-muted-foreground">
                High performance Rust-based proxy engine
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-slate-500 mr-2 hidden md:inline">Logged in as admin</span>
            <Button variant="outline" size="sm" onClick={onLogout}>
              <LogOut className="mr-2 h-4 w-4" /> Logout
            </Button>
          </div>
        </div>

        {/* Tabs Navigation */}
        <Tabs defaultValue="dashboard" className="space-y-4">
          <TabsList>
            <TabsTrigger value="dashboard">Dashboard</TabsTrigger>
            <TabsTrigger value="hosts">Hosts</TabsTrigger>
            <TabsTrigger value="streams">Streams</TabsTrigger>
            <TabsTrigger value="access">Access Lists</TabsTrigger>
            <TabsTrigger value="logs">Logs</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>
          
          <TabsContent value="dashboard" className="space-y-4">
            <DashboardTab />
          </TabsContent>
          
          <TabsContent value="hosts" className="space-y-4">
            <HostsTab />
          </TabsContent>

          <TabsContent value="streams" className="space-y-4">
            <StreamsTab />
          </TabsContent>

          <TabsContent value="access" className="space-y-4">
            <AccessListsTab />
          </TabsContent>

          <TabsContent value="logs" className="space-y-4">
             <LogsTab />
          </TabsContent>

          <TabsContent value="settings" className="space-y-4">
             <SettingsTab />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

export default App;