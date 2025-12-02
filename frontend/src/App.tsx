import { useEffect, useState, useMemo } from "react";
import { Toaster, toast } from "sonner";
import { 
  Plus, Trash2, Server, Lock, LogOut, 
  Search, RefreshCw, Loader2, Globe, ShieldCheck, AlertCircle 
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter, DialogClose,
} from "@/components/ui/dialog";
import {
  Card, CardContent, CardDescription, CardHeader, CardTitle,
} from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";

const API_BASE = "/api"; 

interface Host {
  domain: string;
  target: string;
  scheme: "http" | "https";
  created_at?: string;
}

function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem("token"));
  const [hosts, setHosts] = useState<Host[]>([]);
  const [loading, setLoading] = useState(false); // 전체 데이터 로딩
  const [actionLoading, setActionLoading] = useState(false); // 버튼 액션 로딩
  const [searchQuery, setSearchQuery] = useState("");
  
  // 모달 상태
  const [isOpen, setIsOpen] = useState(false);
  const [newHost, setNewHost] = useState({ domain: "", target: "", scheme: "http" });

  // 로그인 상태
  const [loginId, setLoginId] = useState("");
  const [loginPw, setLoginPw] = useState("");

  // --- API 헬퍼 함수 ---
  const apiCall = async (endpoint: string, options: RequestInit = {}) => {
    const headers = {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options.headers,
    };
    
    try {
      const res = await fetch(`${API_BASE}${endpoint}`, { ...options, headers });
      if (res.status === 401) {
        handleLogout();
        throw new Error("Unauthorized");
      }
      return res;
    } catch (error) {
      throw error;
    }
  };

  const fetchHosts = async () => {
    if (!token) return;
    setLoading(true);
    try {
      const res = await apiCall("/hosts");
      if (!res.ok) throw new Error("Failed to fetch");
      const data = await res.json();
      setHosts(data);
    } catch (err) {
      // 401 에러가 아닐 때만 토스트 띄움
      if (token) toast.error("Failed to load hosts info.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (token) fetchHosts();
  }, [token]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setActionLoading(true);
    try {
      const res = await apiCall("/login", {
        method: "POST",
        body: JSON.stringify({ username: loginId, password: loginPw }),
      });
      
      if (res.ok) {
        const data = await res.json();
        setToken(data.token);
        localStorage.setItem("token", data.token);
        toast.success(`Welcome back, ${loginId}!`);
      } else {
        toast.error("Invalid credentials.");
      }
    } catch (err) {
      toast.error("Login server error.");
    } finally {
      setActionLoading(false);
    }
  };

  const handleLogout = () => {
    setToken(null);
    localStorage.removeItem("token");
    setHosts([]);
    toast.info("Logged out successfully");
  };

  const handleAddHost = async () => {
    // 간단한 유효성 검사
    if (!newHost.domain || !newHost.target) {
      toast.warning("Please fill in all required fields.");
      return;
    }

    setActionLoading(true);
    try {
      const res = await apiCall("/hosts", {
        method: "POST",
        body: JSON.stringify(newHost),
      });

      if (res.ok) {
        toast.success("New proxy host added!");
        setIsOpen(false);
        setNewHost({ domain: "", target: "", scheme: "http" });
        fetchHosts();
      } else {
        const errData = await res.json();
        toast.error(errData.message || "Failed to add host.");
      }
    } catch (err) {
      toast.error("Network error.");
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeleteHost = async (domain: string) => {
    if (!confirm(`Are you sure you want to delete ${domain}?`)) return;

    try {
      const res = await apiCall(`/hosts/${domain}`, { method: "DELETE" });
      if (res.ok) {
        toast.success("Host deleted successfully.");
        // UI에서 즉시 제거 (낙관적 업데이트)
        setHosts((prev) => prev.filter(h => h.domain !== domain));
      } else {
        toast.error("Failed to delete host.");
      }
    } catch (err) {
      toast.error("Error connecting to server.");
    }
  };

  // 검색 필터링 로직
  const filteredHosts = useMemo(() => {
    return hosts.filter(host => 
      host.domain.toLowerCase().includes(searchQuery.toLowerCase()) ||
      host.target.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }, [hosts, searchQuery]);

  // --- 로그인 화면 ---
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
                  disabled={actionLoading}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input 
                  id="password" 
                  type="password" 
                  value={loginPw} 
                  onChange={(e) => setLoginPw(e.target.value)} 
                  disabled={actionLoading}
                />
              </div>
              <Button type="submit" className="w-full" disabled={actionLoading}>
                {actionLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Sign In"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    );
  }

  // --- 메인 대시보드 ---
  return (
    <div className="min-h-screen bg-slate-50/50 p-6 md:p-10">
      <Toaster position="top-right" />
      <div className="max-w-6xl mx-auto space-y-6">
        
        {/* 헤더 섹션 */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="bg-primary text-primary-foreground p-2 rounded-lg">
              <Server className="h-6 w-6" />
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
            <Button variant="outline" size="sm" onClick={handleLogout}>
              <LogOut className="mr-2 h-4 w-4" /> Logout
            </Button>
          </div>
        </div>

        {/* 메인 컨텐츠 카드 */}
        <Card className="shadow-md border-slate-200">
          <CardHeader className="pb-3">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <CardTitle>Proxy Hosts</CardTitle>
                <CardDescription>
                  Active routing rules configured in memory.
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                {/* 검색바 */}
                <div className="relative">
                  <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search domains..."
                    className="pl-8 w-[200px] md:w-[250px]"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>
                {/* 새로고침 버튼 */}
                <Button variant="outline" size="icon" onClick={fetchHosts} disabled={loading}>
                  <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                </Button>
                
                {/* 추가 모달 */}
                <Dialog open={isOpen} onOpenChange={setIsOpen}>
                  <DialogTrigger asChild>
                    <Button>
                      <Plus className="mr-2 h-4 w-4" /> Add Host
                    </Button>
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Add New Proxy Host</DialogTitle>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <Label htmlFor="domain">Domain Names</Label>
                        <Input
                          id="domain"
                          placeholder="e.g. blog.example.com"
                          value={newHost.domain}
                          onChange={(e) => setNewHost({ ...newHost, domain: e.target.value })}
                        />
                        <p className="text-xs text-muted-foreground">
                          Public domain pointing to this server.
                        </p>
                      </div>
                      
                      <div className="grid grid-cols-4 gap-4">
                        <div className="col-span-1 grid gap-2">
                          <Label>Scheme</Label>
                          <Select 
                            value={newHost.scheme} 
                            onValueChange={(v) => setNewHost({...newHost, scheme: v as any})}
                          >
                            <SelectTrigger>
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="http">http://</SelectItem>
                              <SelectItem value="https">https://</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="col-span-3 grid gap-2">
                          <Label htmlFor="target">Forward IP / Hostname</Label>
                          <Input
                            id="target"
                            placeholder="e.g. 127.0.0.1:3000"
                            value={newHost.target}
                            onChange={(e) => setNewHost({ ...newHost, target: e.target.value })}
                          />
                        </div>
                      </div>
                      <div className="bg-yellow-50 text-yellow-700 p-3 rounded text-sm flex gap-2">
                        <AlertCircle className="h-4 w-4 mt-0.5" />
                        Changes are applied instantly to the running engine without restart.
                      </div>
                    </div>
                    <DialogFooter>
                      <DialogClose asChild>
                        <Button variant="outline">Cancel</Button>
                      </DialogClose>
                      <Button onClick={handleAddHost} disabled={actionLoading}>
                        {actionLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Save Route"}
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </div>
            </div>
          </CardHeader>

          <CardContent>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow className="bg-slate-50">
                    <TableHead className="w-[300px]">Source Domain</TableHead>
                    <TableHead>Destination</TableHead>
                    <TableHead>Scheme</TableHead>
                    <TableHead>Security</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loading && hosts.length === 0 ? (
                     <TableRow>
                       <TableCell colSpan={5} className="h-24 text-center">
                         <div className="flex justify-center items-center gap-2 text-muted-foreground">
                           <Loader2 className="h-4 w-4 animate-spin" /> Loading configurations...
                         </div>
                       </TableCell>
                     </TableRow>
                  ) : filteredHosts.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={5} className="h-32 text-center text-muted-foreground">
                        {searchQuery 
                          ? "No hosts match your search." 
                          : "No proxy hosts configured yet. Click 'Add Host' to start."}
                      </TableCell>
                    </TableRow>
                  ) : (
                    filteredHosts.map((host) => (
                      <TableRow key={host.domain} className="hover:bg-slate-50/50">
                        <TableCell className="font-medium">
                          <div className="flex items-center gap-2">
                            <Globe className="h-4 w-4 text-slate-400" />
                            <a 
                              href={`http://${host.domain}`} 
                              target="_blank" 
                              rel="noreferrer"
                              className="hover:underline hover:text-primary transition-colors"
                            >
                              {host.domain}
                            </a>
                          </div>
                        </TableCell>
                        <TableCell className="font-mono text-sm text-slate-600">
                          {host.target}
                        </TableCell>
                        <TableCell>
                           <Badge variant="outline" className="uppercase text-xs font-bold">
                             {host.scheme}
                           </Badge>
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-1.5 text-sm text-slate-600">
                            {host.scheme === 'https' ? (
                                <>
                                  <ShieldCheck className="h-4 w-4 text-green-500" />
                                  <span className="text-green-700">Encrypted</span>
                                </>
                            ) : (
                                <span className="text-slate-400">Standard</span>
                            )}
                          </div>
                        </TableCell>
                        <TableCell className="text-right">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="text-red-500 hover:text-red-600 hover:bg-red-50"
                            onClick={() => handleDeleteHost(host.domain)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default App;