import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Edit, RefreshCw, CornerDownRight, ArrowRightLeft, Trash2, Plus } from "lucide-react";
import type { Host, Location, Header } from "@/hooks/useHosts";
import { toast } from "sonner";
import { useAddHost, useAddLocation, useDeleteLocation, useAddHostHeader, useDeleteHostHeader } from "@/hooks/useHosts";
import { useAccessLists } from "@/hooks/useAccessLists";

interface EditHostDialogProps {
  host: Host | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function EditHostDialog({ host, open, onOpenChange }: EditHostDialogProps) {
  const addHostMutation = useAddHost();
  const addLocationMutation = useAddLocation();
  const deleteLocationMutation = useDeleteLocation();
  const addHeaderMutation = useAddHostHeader();
  const deleteHeaderMutation = useDeleteHostHeader();
  const { data: accessLists } = useAccessLists();

  const [editFormHost, setEditFormHost] = useState<Partial<Host>>({});
  const [newLocation, setNewLocation] = useState<Location>({ path: "/", target: "", scheme: "http", rewrite: false });
  const [newHeader, setNewHeader] = useState<Omit<Header, 'id'>>({ name: "", value: "", target: "request" });

  useEffect(() => {
    if (host) {
        setEditFormHost({
            target: host.target,
            scheme: host.scheme,
            ssl_forced: host.ssl_forced,
            redirect_to: host.redirect_to || "",
            redirect_status: host.redirect_status || 301,
            access_list_id: host.access_list_id || null,
        });
    }
  }, [host]);

  // --- Multi-Target Logic for Host Settings ---
  const getTargets = () => {
    if (!editFormHost.target) return [""];
    return editFormHost.target.split(',').map(t => t.trim());
  };

  const setTargets = (targets: string[]) => {
    setEditFormHost({ ...editFormHost, target: targets.join(',') });
  };

  const handleAddTarget = () => {
    const current = getTargets();
    setTargets([...current, ""]);
  };

  const handleRemoveTarget = (index: number) => {
    const current = getTargets();
    if (current.length <= 1) {
        setTargets([""]); // Reset if last one
        return;
    }
    const next = current.filter((_, i) => i !== index);
    setTargets(next);
  };

  const handleTargetChange = (index: number, value: string) => {
    const current = getTargets();
    current[index] = value;
    setTargets(current);
  };
  // --------------------------------------------

  const handleUpdateHost = () => {
    if (!host) return;
    
    const hostPayload = {
      ...editFormHost,
      domain: host.domain, // 도메인은 변경 불가
      redirect_to: editFormHost.redirect_to || null,
      access_list_id: editFormHost.access_list_id === 0 ? null : editFormHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        toast.success("Host updated successfully");
        onOpenChange(false);
      }
    });
  };

  const handleAddLocation = () => {
    if (!host || !newLocation.path || !newLocation.target) return;
    addLocationMutation.mutate({ domain: host.domain, location: newLocation }, {
        onSuccess: () => {
            setNewLocation({ path: "/", target: "", scheme: "http", rewrite: false });
        }
    });
  };

  const handleDeleteLocation = (path: string) => {
    if (!host) return;
    deleteLocationMutation.mutate({ domain: host.domain, path });
  };

  const handleAddHeader = () => {
    if (!host || !newHeader.name || !newHeader.value) return toast.warning("Header name and value are required");
    addHeaderMutation.mutate({ domain: host.domain, header: newHeader }, {
        onSuccess: () => {
            setNewHeader({ name: "", value: "", target: "request" });
            toast.success("Header added successfully");
        },
        onError: (error) => {
            toast.error(`Failed to add header: ${error.message}`);
        }
    });
  };

  const handleDeleteHeader = (headerId: number) => {
    if (!host) return;
    if (!confirm("Are you sure you want to delete this header?")) return;
    deleteHeaderMutation.mutate({ domain: host.domain, headerId }, {
        onSuccess: () => {
            toast.success("Header deleted successfully");
        },
        onError: (error) => {
            toast.error(`Failed to delete header: ${error.message}`);
        }
    });
  };


  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Edit className="h-5 w-5" />
            Edit Host: {host?.domain}
          </DialogTitle>
          <DialogDescription>
            Modify host settings and manage path-based routing, and custom headers.
          </DialogDescription>
        </DialogHeader>
        
        <Tabs defaultValue="settings" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="settings">Host Settings</TabsTrigger>
            <TabsTrigger value="locations">Locations ({host?.locations?.length || 0})</TabsTrigger>
            <TabsTrigger value="headers">Custom Headers ({host?.headers?.length || 0})</TabsTrigger>
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
              
              {/* Multi-Target Input for Edit */}
              <div className="grid gap-2">
                <div className="flex items-center justify-between">
                    <Label>Target(s)</Label>
                    <Button type="button" variant="ghost" size="sm" onClick={handleAddTarget} className="h-6 px-2 text-xs">
                        <Plus className="h-3 w-3 mr-1" /> Add
                    </Button>
                </div>
                <div className="space-y-2 max-h-[150px] overflow-y-auto pr-1">
                    {getTargets().map((t, idx) => (
                        <div key={idx} className="flex gap-2">
                            <Input 
                                value={t} 
                                onChange={e => handleTargetChange(idx, e.target.value)} 
                                placeholder={idx === 0 ? "127.0.0.1:8080" : "10.0.0.2:8080"} 
                                className="flex-1"
                            />
                            {getTargets().length > 1 && (
                                <Button type="button" variant="ghost" size="icon" onClick={() => handleRemoveTarget(idx)}>
                                    <Trash2 className="h-4 w-4 text-red-500" />
                                </Button>
                            )}
                        </div>
                    ))}
                </div>
                <p className="text-[10px] text-muted-foreground">
                    Multiple targets enable <strong>Load Balancing</strong> (Random).
                </p>
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
                <Badge variant="outline" className="ml-2 text-[10px] bg-yellow-50 text-yellow-700 border-yellow-200">
                  HTTPS Only
                </Badge>
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
                  <Label>Target(s)</Label>
                  {/* Note: Ideally we should use the same multi-target input for Locations too, but for now keeping it simple as CSV string input. I'll just update placeholder */}
                  <Input value={newLocation.target} onChange={e => setNewLocation({...newLocation, target: e.target.value})} placeholder="10.0.0.5:3000, 10.0.0.6:3000" />
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
              {host?.locations?.length === 0 && (
                <div className="text-sm text-muted-foreground italic">No extra paths configured. All traffic goes to default target.</div>
              )}
              {host?.locations?.map((loc) => (
                <div key={loc.path} className="flex items-center justify-between p-2 bg-slate-50 rounded border">
                  <div className="flex items-center gap-3">
                    <CornerDownRight className="h-4 w-4 text-slate-400" />
                    <Badge variant="outline">{loc.path}</Badge>
                    <span className="text-sm text-slate-500">forwards to</span>
                    <span className="font-mono text-sm">{loc.scheme}://{loc.target}</span>
                    {loc.rewrite && (
                      <Badge variant="secondary" className="text-[10px] bg-yellow-100 text-yellow-800 hover:bg-yellow-100">
                        <ArrowRightLeft className="mr-1 h-3 w-3 inline" /> Stripped
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

          {/* Custom Headers Tab */}
          <TabsContent value="headers" className="space-y-4 pt-4">
            <div className="grid gap-4 border-b pb-4">
                <div className="grid grid-cols-2 gap-4">
                    <div className="grid gap-2">
                        <Label>Header Name</Label>
                        <Input 
                            value={newHeader.name} 
                            onChange={e => setNewHeader({...newHeader, name: e.target.value})} 
                            placeholder="X-Custom-Header" 
                        />
                    </div>
                    <div className="grid gap-2">
                        <Label>Header Value</Label>
                        <Input 
                            value={newHeader.value} 
                            onChange={e => setNewHeader({...newHeader, value: e.target.value})} 
                            placeholder="my-value" 
                        />
                    </div>
                </div>
                <div className="flex items-end gap-3">
                    <div className="grid gap-2 w-[150px]">
                        <Label>Target</Label>
                        <Select 
                            value={newHeader.target} 
                            onValueChange={v => setNewHeader({...newHeader, target: v as "request" | "response"})}
                        >
                            <SelectTrigger><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="request">Request</SelectItem>
                                <SelectItem value="response">Response</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <Button className="ml-auto" onClick={handleAddHeader} disabled={addHeaderMutation.isPending}>
                        <Plus className="h-4 w-4 mr-2" /> Add Header
                    </Button>
                </div>
            </div>

            <div className="space-y-2">
                <Label>Configured Headers</Label>
                {host?.headers?.length === 0 && (
                    <div className="text-sm text-muted-foreground italic">No custom headers configured.</div>
                )}
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Name</TableHead>
                            <TableHead>Value</TableHead>
                            <TableHead>Target</TableHead>
                            <TableHead className="w-[80px]">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {host?.headers?.map((header) => (
                            <TableRow key={header.id}>
                                <TableCell className="font-mono text-sm">{header.name}</TableCell>
                                <TableCell className="font-mono text-sm">{header.value}</TableCell>
                                <TableCell className="capitalize">{header.target}</TableCell>
                                <TableCell>
                                    <Button variant="ghost" size="sm" onClick={() => handleDeleteHeader(header.id)} disabled={deleteHeaderMutation.isPending}>
                                        <Trash2 className="h-4 w-4 text-red-500" />
                                    </Button>
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </div>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
