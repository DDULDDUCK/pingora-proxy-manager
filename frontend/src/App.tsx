import { useState, createContext, useContext } from "react";
import { Toaster, toast } from "sonner";
import { useTranslation } from "react-i18next";
import {
  Lock, LogOut, Loader2
} from "lucide-react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from "@/components/ui/card";

import { DashboardTab } from "@/components/dashboard/DashboardTab";
import { HostsTab } from "@/components/hosts/HostsTab";
import { StreamsTab } from "@/components/streams/StreamsTab";
import { LogsTab } from "@/components/dashboard/LogsTab";
import { SettingsTab } from "@/components/dashboard/SettingsTab";
import { AccessListsTab } from "@/components/access/AccessListsTab";
import { UsersTab } from "@/components/users/UsersTab";
import { AuditLogsTab } from "@/components/audit/AuditLogsTab";
import { CertificatesTab } from "@/components/dns/CertificatesTab";
import { LanguageSwitcher } from "@/components/layout/LanguageSwitcher";
import { useCurrentUser, type User } from "@/hooks/useUsers";
import { api } from "@/lib/api";
import ppnIcon from '@/assets/ppnicon.png';

// 권한 컨텍스트
interface AuthContextType {
  user: User | null | undefined;
  isAdmin: boolean;
  canManageHosts: boolean;
  isLoading: boolean;
}

const AuthContext = createContext<AuthContextType>({
  user: null,
  isAdmin: false,
  canManageHosts: false,
  isLoading: true,
});

export const useAuth = () => useContext(AuthContext);

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  const { t } = useTranslation();
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
        toast.success(t('login.welcomeBack', { username: loginId }));
      } else {
        toast.error(t('login.invalidCredentials'));
      }
    } catch {
      toast.error(t('login.serverError'));
    } finally {
      setLoginLoading(false);
    }
  };

  const handleLogout = () => {
    api.removeToken();
    setToken(null);
    toast.info(t('login.loggedOut'));
  };

  if (!token) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-100 p-4">
        <Toaster position="top-center" />
        <div className="absolute top-4 right-4">
          <LanguageSwitcher />
        </div>
        <Card className="w-full max-w-sm shadow-lg">
          <CardHeader className="text-center">
            <div className="mx-auto bg-primary/10 p-3 rounded-full w-fit mb-2">
              <Lock className="h-6 w-6 text-primary" />
            </div>
            <CardTitle className="text-xl">{t('app.title')}</CardTitle>
            <CardDescription>{t('app.subtitle')}</CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="username">{t('login.username')}</Label>
                <Input 
                  id="username" 
                  value={loginId} 
                  onChange={(e) => setLoginId(e.target.value)} 
                  disabled={loginLoading}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">{t('login.password')}</Label>
                <Input 
                  id="password" 
                  type="password" 
                  value={loginPw} 
                  onChange={(e) => setLoginPw(e.target.value)} 
                  disabled={loginLoading}
                />
              </div>
              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : t('login.signIn')}
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
  const { t } = useTranslation();
  const { data: user, isLoading: userLoading } = useCurrentUser();
  
  const isAdmin = user?.role === "admin";
  const canManageHosts = user?.role === "admin" || user?.role === "operator";
  
  const roleColors: Record<string, string> = {
    admin: "bg-red-100 text-red-700 border-red-200",
    operator: "bg-blue-100 text-blue-700 border-blue-200",
    viewer: "bg-gray-100 text-gray-700 border-gray-200",
  };

  return (
    <AuthContext.Provider value={{ user, isAdmin, canManageHosts, isLoading: userLoading }}>
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
                  {t('app.mainTitle')}
                </h1>
                <p className="text-sm text-muted-foreground">
                  {t('app.mainSubtitle')}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <LanguageSwitcher />
              {user && (
                <div className="flex items-center gap-2 mr-2">
                  <span className="text-sm text-slate-500 hidden md:inline">
                    {user.username}
                  </span>
                  <Badge variant="outline" className={roleColors[user.role] || roleColors.viewer}>
                    {user.role.toUpperCase()}
                  </Badge>
                </div>
              )}
              <Button variant="outline" size="sm" onClick={onLogout}>
                <LogOut className="mr-2 h-4 w-4" /> {t('app.logout')}
              </Button>
            </div>
          </div>

          {/* Tabs Navigation */}
          <Tabs defaultValue="dashboard" className="space-y-4">
            <TabsList className="flex flex-wrap">
              <TabsTrigger value="dashboard">{t('tabs.dashboard')}</TabsTrigger>
              <TabsTrigger value="hosts">{t('tabs.hosts')}</TabsTrigger>
              <TabsTrigger value="streams">{t('tabs.streams')}</TabsTrigger>
              <TabsTrigger value="access">{t('tabs.accessLists')}</TabsTrigger>
              {canManageHosts && <TabsTrigger value="certs">{t('tabs.certificates')}</TabsTrigger>}
              {isAdmin && <TabsTrigger value="users">{t('tabs.users')}</TabsTrigger>}
              <TabsTrigger value="audit">{t('tabs.auditLogs')}</TabsTrigger>
              {canManageHosts && <TabsTrigger value="logs">{t('tabs.logs')}</TabsTrigger>}
              {isAdmin && <TabsTrigger value="settings">{t('tabs.settings')}</TabsTrigger>}
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

            {canManageHosts && (
              <TabsContent value="certs" className="space-y-4">
                <CertificatesTab />
              </TabsContent>
            )} {/* Added Content */}

            {isAdmin && (
              <TabsContent value="users" className="space-y-4">
                <UsersTab />
              </TabsContent>
            )}

            <TabsContent value="audit" className="space-y-4">
              <AuditLogsTab />
            </TabsContent>

            {canManageHosts && (
              <TabsContent value="logs" className="space-y-4">
                 <LogsTab />
              </TabsContent>
            )}

            {isAdmin && (
              <TabsContent value="settings" className="space-y-4">
                 <SettingsTab />
              </TabsContent>
            )}
          </Tabs>
        </div>
      </div>
    </AuthContext.Provider>
  );
}

export default App;