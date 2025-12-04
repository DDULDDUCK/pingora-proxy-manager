import { useState, useMemo } from "react";
import { RefreshCw, Plus, Settings, ShieldCheck, Trash2, CornerDownRight, ArrowRightLeft, Link2, Lock, Edit } from "lucide-react";
import { useHosts, useAddHost, useDeleteHost, useAddLocation, useDeleteLocation, useIssueCert } from "@/hooks/useHosts";
import { useAccessLists } from "@/hooks/useAccessLists";
import { useAuth } from "@/App";
import type { Host, Location } from "@/hooks/useHosts";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Textarea } from "@/components/ui/textarea";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Card, CardContent, CardHeader, CardTitle, CardDescription,
} from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import {
  Tabs, TabsContent, TabsList, TabsTrigger,
} from "@/components/ui/tabs";
import { toast } from "sonner";

export function HostsTab() {
  const { canManageHosts } = useAuth();
  const { data: hosts, isLoading, refetch } = useHosts();
  const { data: accessLists } = useAccessLists();
  
  const addHostMutation = useAddHost();
  const deleteHostMutation = useDeleteHost();
  const addLocationMutation = useAddLocation();
  const deleteLocationMutation = useDeleteLocation();
  const issueCertMutation = useIssueCert();

  const [searchQuery, setSearchQuery] = useState("");
  
  // Add Host Dialog (다중 도메인 지원)
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [bulkMode, setBulkMode] = useState(false);
  const [bulkDomains, setBulkDomains] = useState("");
  const [newHost, setNewHost] = useState<Partial<Host>>({
      domain: "",
      target: "",
      scheme: "http",
      ssl_forced: false,
      redirect_to: "",
      redirect_status: 301,
      access_list_id: null
  });

  // Edit Host Dialog (호스트 정보 수정 + 로케이션 관리)
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<Host | null>(null);
  const [editFormHost, setEditFormHost] = useState<Partial<Host>>({});
  const [newLocation, setNewLocation] = useState<Location>({ path: "/", target: "", scheme: "http", rewrite: false });

  const handleAddHost = () => {
    // 다중 도메인 모드
    if (bulkMode) {
      const domains = bulkDomains
        .split(/[,\n]/)
        .map(d => d.trim())
        .filter(d => d.length > 0);
      
      if (domains.length === 0) return toast.warning("At least one domain is required");
      
      // 각 도메인에 대해 호스트 생성
      let created = 0;
      domains.forEach((domain) => {
        const hostPayload = {
          ...newHost,
          domain,
          target: newHost.target || "127.0.0.1:8080",
          redirect_to: newHost.redirect_to || null,
          access_list_id: newHost.access_list_id === 0 ? null : newHost.access_list_id
        };
        
        addHostMutation.mutate(hostPayload, {
          onSuccess: () => {
            created++;
            if (created === domains.length) {
              setIsAddOpen(false);
              setBulkDomains("");
              setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, redirect_to: "", redirect_status: 301, access_list_id: null });
              toast.success(`${created} hosts created successfully`);
            }
          }
        });
      });
      return;
    }
    
    // 단일 도메인 모드
    if (!newHost.domain) return toast.warning("Domain is required");
    
    const hostPayload = {
        ...newHost,
        target: newHost.target || "127.0.0.1:8080",
        redirect_to: newHost.redirect_to || null,
        access_list_id: newHost.access_list_id === 0 ? null : newHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        setIsAddOpen(false);
        setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, redirect_to: "", redirect_status: 301, access_list_id: null });
      }
    });
  };

  const handleUpdateHost = () => {
    if (!editingHost) return;
    
    const hostPayload = {
      ...editFormHost,
      domain: editingHost.domain, // 도메인은 변경 불가
      redirect_to: editFormHost.redirect_to || null,
      access_list_id: editFormHost.access_list_id === 0 ? null : editFormHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        toast.success("Host updated successfully");
      }
    });
  };

  const handleDeleteHost = (domain: string) => {
    if (!confirm(`Delete ${domain}?`)) return;
    deleteHostMutation.mutate(domain);
  };

  const handleIssueCert = (domain: string) => {
    issueCertMutation.mutate({ domain, email: "admin@example.com" });
  };

  const handleAddLocation = () => {
    if (!editingHost || !newLocation.path || !newLocation.target) return;
    addLocationMutation.mutate({ domain: editingHost.domain, location: newLocation }, {
        onSuccess: () => {
            setNewLocation({ path: "/", target: "", scheme: "http", rewrite: false });
        }
    });
  };

  const handleDeleteLocation = (path: string) => {
    if (!editingHost) return;
    deleteLocationMutation.mutate({ domain: editingHost.domain, path });
  };

  // Update editingHost when hosts data changes
  useMemo(() => {
      if (editingHost && hosts) {
          const updated = hosts.find(h => h.domain === editingHost.domain);
          if (updated) setEditingHost(updated);
      }
  }, [hosts, editingHost]);

  const openEdit = (host: Host) => {
    setEditingHost(host);
    setEditFormHost({
      target: host.target,
      scheme: host.scheme,
      ssl_forced: host.ssl_forced,
      redirect_to: host.redirect_to || "",
      redirect_status: host.redirect_status || 301,
      access_list_id: host.access_list_id || null
    });
    setIsEditOpen(true);
  };

  const filteredHosts = useMemo(() => (hosts || []).filter(h => 
    h.domain.toLowerCase().includes(searchQuery.toLowerCase())
  ), [hosts, searchQuery]);

  return (
    <Card>
       <CardHeader className="pb-3">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <CardTitle>Proxy Hosts</CardTitle>
                <CardDescription>Manage your routing and redirection rules.</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  placeholder="Search domains..."
                  className="w-[200px]"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
                <Button variant="outline" size="icon" onClick={() => refetch()} disabled={isLoading}>
                  <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                </Button>
                {canManageHosts && (
                <Dialog open={isAddOpen} onOpenChange={(open) => { setIsAddOpen(open); if (!open) setBulkMode(false); }}>
                  <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" /> Add Host</Button></DialogTrigger>
                  <DialogContent className="max-w-lg">
                    <DialogHeader>
                      <DialogTitle>Add New Host</DialogTitle>
                      <DialogDescription>Add one or multiple proxy hosts at once</DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <div className="flex items-center justify-between">
                          <Label>Domain{bulkMode ? "s" : ""}</Label>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => setBulkMode(!bulkMode)}
                            className="text-xs"
                          >
                            {bulkMode ? "Single Mode" : "Bulk Mode"}
                          </Button>
                        </div>
                        {bulkMode ? (
                          <Textarea
                            value={bulkDomains}
                            onChange={e => setBulkDomains(e.target.value)}
                            placeholder="Enter multiple domains (comma or newline separated):&#10;example1.com&#10;example2.com&#10;example3.com"
                            rows={4}
                          />
                        ) : (
                          <Input value={newHost.domain} onChange={e => setNewHost({...newHost, domain: e.target.value})} placeholder="example.com" />
                        )}
                        {bulkMode && (
                          <p className="text-xs text-muted-foreground">
                            All domains will share the same target, scheme, and other settings.
                          </p>
                        )}
                      </div>
                      
                      <div className="grid grid-cols-2 gap-4">
                          <div className="grid gap-2">
                             <Label>Scheme</Label>
                             <Select value={newHost.scheme} onValueChange={v => setNewHost({...newHost, scheme: v as any})}>
                               <SelectTrigger><SelectValue /></SelectTrigger>
                               <SelectContent>
                                 <SelectItem value="http">http://</SelectItem>
                                 <SelectItem value="https">https://</SelectItem>
                               </SelectContent>
                             </Select>
                          </div>
                          <div className="grid gap-2">
                            <Label>Target</Label>
                            <Input value={newHost.target} onChange={e => setNewHost({...newHost, target: e.target.value})} placeholder="127.0.0.1:8080" />
                          </div>
                      </div>

                      <div className="grid gap-2">
                          <Label>Access List</Label>
                          <Select value={newHost.access_list_id?.toString() || "0"} onValueChange={v => setNewHost({...newHost, access_list_id: parseInt(v)})}>
                              <SelectTrigger><SelectValue placeholder="Public (No Auth)" /></SelectTrigger>
                              <SelectContent>
                                  <SelectItem value="0">Public (No Auth)</SelectItem>
                                  {accessLists?.map(acl => (
                                      <SelectItem key={acl.id} value={acl.id.toString()}>{acl.name}</SelectItem>
                                  ))}
                              </SelectContent>
                          </Select>
                      </div>

                      <div className="flex items-center space-x-2 pt-2">
                          <input 
                            type="checkbox" 
                            id="ssl_forced" 
                            className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                            checked={newHost.ssl_forced || false}
                            onChange={e => setNewHost({...newHost, ssl_forced: e.target.checked})}
                          />
                          <Label htmlFor="ssl_forced" className="cursor-pointer flex items-center">
                              Force SSL
                              <Badge variant="outline" className="ml-2 text-[10px] bg-yellow-50 text-yellow-700 border-yellow-200">HTTPS Only</Badge>
                          </Label>
                      </div>

                      <div className="border-t pt-4 mt-2">
                          <Label className="text-muted-foreground mb-2 block">Redirection (Optional)</Label>
                          <div className="grid grid-cols-3 gap-4">
                              <div className="col-span-2 grid gap-2">
                                  <Label className="text-xs">Redirect To URL</Label>
                                  <Input value={newHost.redirect_to || ""} onChange={e => setNewHost({...newHost, redirect_to: e.target.value})} placeholder="https://google.com" />
                              </div>
                              <div className="grid gap-2">
                                  <Label className="text-xs">Status Code</Label>
                                  <Select value={newHost.redirect_status?.toString()} onValueChange={v => setNewHost({...newHost, redirect_status: parseInt(v)})}>
                                    <SelectTrigger><SelectValue /></SelectTrigger>
                                    <SelectContent>
                                      <SelectItem value="301">301 (Perm)</SelectItem>
                                      <SelectItem value="302">302 (Temp)</SelectItem>
                                    </SelectContent>
                                  </Select>
                              </div>
                          </div>
                      </div>
                    </div>
                    <DialogFooter>
                       <Button onClick={handleAddHost} disabled={addHostMutation.isPending}>
                           {addHostMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                           Save
                       </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
                )}
              </div>
            </div>
       </CardHeader>
       <CardContent>
          <Table>
             <TableHeader>
               <TableRow>
                 <TableHead>Domain</TableHead>
                 <TableHead>Destination</TableHead>
                 <TableHead>Status</TableHead>
                 <TableHead className="text-right">Actions</TableHead>
               </TableRow>
             </TableHeader>
             <TableBody>
               {filteredHosts.map(host => (
                 <TableRow key={host.domain}>
                    <TableCell className="font-medium">
                        <div className="flex flex-col">
                            <span>{host.domain}</span>
                            <div className="flex gap-1 mt-1">
                                {host.ssl_forced && <Badge variant="outline" className="text-[10px] border-green-200 text-green-700 bg-green-50"><ShieldCheck className="h-3 w-3 mr-1"/> SSL</Badge>}
                                {host.access_list_id && <Badge variant="outline" className="text-[10px] border-orange-200 text-orange-700 bg-orange-50"><Lock className="h-3 w-3 mr-1"/> ACL</Badge>}
                            </div>
                        </div>
                    </TableCell>
                    <TableCell>
                        {host.redirect_to ? (
                            <div className="flex items-center text-blue-600">
                                <Link2 className="h-3 w-3 mr-1" />
                                <span className="text-xs font-mono">{host.redirect_to}</span>
                                <Badge variant="secondary" className="ml-2 text-[10px]">{host.redirect_status}</Badge>
                            </div>
                        ) : (
                            <div className="flex items-center">
                                <span className="text-sm">{host.target}</span>
                                <Badge variant="outline" className="ml-2 text-[10px]">{host.scheme}</Badge>
                            </div>
                        )}
                    </TableCell>
                    <TableCell>
                      {host.locations && host.locations.length > 0 ? (
                        <div className="flex flex-wrap gap-1">
                          {host.locations.map(l => (
                            <Badge key={l.path} variant="secondary" className="text-[10px]">
                              {l.path} → {l.target}
                              {l.rewrite && <ArrowRightLeft className="ml-1 h-3 w-3 inline" />}
                            </Badge>
                          ))}
                        </div>
                      ) : (
                        <span className="text-muted-foreground text-xs">Default only</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        {canManageHosts && (
                          <Button variant="ghost" size="sm" onClick={() => openEdit(host)}>
                            <Settings className="h-4 w-4 text-slate-500" />
                          </Button>
                        )}
                        {canManageHosts && (
                          <Button variant="ghost" size="sm" onClick={() => handleIssueCert(host.domain)}>
                            <ShieldCheck className="h-4 w-4 text-blue-500" />
                          </Button>
                        )}
                        {canManageHosts && (
                          <Button variant="ghost" size="sm" onClick={() => handleDeleteHost(host.domain)}>
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        )}
                      </div>
                    </TableCell>
                 </TableRow>
               ))}
             </TableBody>
          </Table>
       </CardContent>

       {/* Edit Dialog - 호스트 정보 수정 + 로케이션 관리 */}
       <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
         <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
           <DialogHeader>
             <DialogTitle className="flex items-center gap-2">
               <Edit className="h-5 w-5" />
               Edit Host: {editingHost?.domain}
             </DialogTitle>
             <DialogDescription>
               Modify host settings and manage path-based routing
             </DialogDescription>
           </DialogHeader>
           
           <Tabs defaultValue="settings" className="w-full">
             <TabsList className="grid w-full grid-cols-2">
               <TabsTrigger value="settings">Host Settings</TabsTrigger>
               <TabsTrigger value="locations">Locations ({editingHost?.locations?.length || 0})</TabsTrigger>
             </TabsList>
             
             {/* Host Settings Tab */}
             <TabsContent value="settings" className="space-y-4 pt-4">
               <div className="grid grid-cols-2 gap-4">
                 <div className="grid gap-2">
                   <Label>Scheme</Label>
                   <Select value={editFormHost.scheme} onValueChange={v => setEditFormHost({...editFormHost, scheme: v as any})}>
                     <SelectTrigger><SelectValue /></SelectTrigger>
                     <SelectContent>
                       <SelectItem value="http">http://</SelectItem>
                       <SelectItem value="https">https://</SelectItem>
                     </SelectContent>
                   </Select>
                 </div>
                 <div className="grid gap-2">
                   <Label>Target</Label>
                   <Input value={editFormHost.target || ""} onChange={e => setEditFormHost({...editFormHost, target: e.target.value})} placeholder="127.0.0.1:8080" />
                 </div>
               </div>

               <div className="grid gap-2">
                 <Label>Access List</Label>
                 <Select value={editFormHost.access_list_id?.toString() || "0"} onValueChange={v => setEditFormHost({...editFormHost, access_list_id: parseInt(v)})}>
                   <SelectTrigger><SelectValue placeholder="Public (No Auth)" /></SelectTrigger>
                   <SelectContent>
                     <SelectItem value="0">Public (No Auth)</SelectItem>
                     {accessLists?.map(acl => (
                       <SelectItem key={acl.id} value={acl.id.toString()}>{acl.name}</SelectItem>
                     ))}
                   </SelectContent>
                 </Select>
               </div>

               <div className="flex items-center space-x-2 pt-2">
                 <input
                   type="checkbox"
                   id="edit_ssl_forced"
                   className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                   checked={editFormHost.ssl_forced || false}
                   onChange={e => setEditFormHost({...editFormHost, ssl_forced: e.target.checked})}
                 />
                 <Label htmlFor="edit_ssl_forced" className="cursor-pointer flex items-center">
                   Force SSL
                   <Badge variant="outline" className="ml-2 text-[10px] bg-yellow-50 text-yellow-700 border-yellow-200">HTTPS Only</Badge>
                 </Label>
               </div>

               <div className="border-t pt-4 mt-2">
                 <Label className="text-muted-foreground mb-2 block">Redirection (Optional)</Label>
                 <div className="grid grid-cols-3 gap-4">
                   <div className="col-span-2 grid gap-2">
                     <Label className="text-xs">Redirect To URL</Label>
                     <Input value={editFormHost.redirect_to || ""} onChange={e => setEditFormHost({...editFormHost, redirect_to: e.target.value})} placeholder="https://google.com" />
                   </div>
                   <div className="grid gap-2">
                     <Label className="text-xs">Status Code</Label>
                     <Select value={editFormHost.redirect_status?.toString()} onValueChange={v => setEditFormHost({...editFormHost, redirect_status: parseInt(v)})}>
                       <SelectTrigger><SelectValue /></SelectTrigger>
                       <SelectContent>
                         <SelectItem value="301">301 (Perm)</SelectItem>
                         <SelectItem value="302">302 (Temp)</SelectItem>
                       </SelectContent>
                     </Select>
                   </div>
                 </div>
               </div>

               <DialogFooter className="pt-4">
                 <Button onClick={handleUpdateHost} disabled={addHostMutation.isPending}>
                   {addHostMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                   Save Changes
                 </Button>
               </DialogFooter>
             </TabsContent>
             
             {/* Locations Tab */}
             <TabsContent value="locations" className="space-y-4 pt-4">
               {/* Add Location Form */}
               <div className="grid gap-4 border-b pb-4">
                 <div className="flex items-end gap-3">
                   <div className="grid gap-1 flex-1">
                     <Label>Path Prefix</Label>
                     <Input value={newLocation.path} onChange={e => setNewLocation({...newLocation, path: e.target.value})} placeholder="/api" />
                   </div>
                   <div className="grid gap-1 flex-1">
                     <Label>Target</Label>
                     <Input value={newLocation.target} onChange={e => setNewLocation({...newLocation, target: e.target.value})} placeholder="10.0.0.5:3000" />
                   </div>
                 </div>
                 <div className="flex items-end gap-3">
                   <div className="grid gap-1 w-[100px]">
                     <Label>Scheme</Label>
                     <Select value={newLocation.scheme} onValueChange={v => setNewLocation({...newLocation, scheme: v as any})}>
                       <SelectTrigger><SelectValue /></SelectTrigger>
                       <SelectContent>
                         <SelectItem value="http">http</SelectItem>
                         <SelectItem value="https">https://</SelectItem>
                       </SelectContent>
                     </Select>
                   </div>
                   <div className="flex items-center space-x-2 pb-2.5">
                     <input
                       type="checkbox"
                       id="rewrite"
                       className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                       checked={newLocation.rewrite || false}
                       onChange={e => setNewLocation({...newLocation, rewrite: e.target.checked})}
                     />
                     <Label htmlFor="rewrite" className="cursor-pointer">Strip Path</Label>
                   </div>
                   <Button className="ml-auto" onClick={handleAddLocation} disabled={addLocationMutation.isPending}>
                     <Plus className="h-4 w-4 mr-2" /> Add
                   </Button>
                 </div>
               </div>

               {/* Locations List */}
               <div className="space-y-2">
                 <Label>Configured Locations</Label>
                 {editingHost?.locations?.length === 0 && (
                   <div className="text-sm text-muted-foreground italic">No extra paths configured. All traffic goes to default target.</div>
                 )}
                 {editingHost?.locations?.map((loc) => (
                   <div key={loc.path} className="flex items-center justify-between p-2 bg-slate-50 rounded border">
                     <div className="flex items-center gap-3">
                       <CornerDownRight className="h-4 w-4 text-slate-400" />
                       <Badge variant="outline">{loc.path}</Badge>
                       <span className="text-sm text-slate-500">forwards to</span>
                       <span className="font-mono text-sm">{loc.scheme}://{loc.target}</span>
                       {loc.rewrite && (
                         <Badge variant="secondary" className="text-[10px] bg-yellow-100 text-yellow-800 hover:bg-yellow-100">
                           Stripped
                         </Badge>
                       )}
                     </div>
                     <Button variant="ghost" size="sm" onClick={() => handleDeleteLocation(loc.path)} disabled={deleteLocationMutation.isPending}>
                       <Trash2 className="h-4 w-4 text-red-500" />
                     </Button>
                   </div>
                 ))}
               </div>
             </TabsContent>
           </Tabs>
         </DialogContent>
       </Dialog>
    </Card>
  );
}