import { useState } from "react";
import { RefreshCw, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useStreams, useAddStream, useDeleteStream } from "@/hooks/useHosts";
import { useAuth } from "@/App";
import type { Stream } from "@/hooks/useHosts";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter,
} from "@/components/ui/dialog";
import {
  Card, CardContent, CardHeader, CardTitle, CardDescription,
} from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";

export function StreamsTab() {
  const { t } = useTranslation();
  const { canManageHosts } = useAuth();
  const { data: streams, isLoading, refetch } = useStreams();
  const addStreamMutation = useAddStream();
  const deleteStreamMutation = useDeleteStream();

  const [isAddOpen, setIsAddOpen] = useState(false);
  const [newStream, setNewStream] = useState<Partial<Stream>>({ 
      listen_port: 8000, 
      forward_host: "", 
      forward_port: 3306,
      protocol: "tcp" 
  });

  const handleAddStream = () => {
    if (!newStream.forward_host) return toast.warning(t('streams.forwardHostRequired'));
    addStreamMutation.mutate(newStream, {
      onSuccess: () => {
        setIsAddOpen(false);
        setNewStream({ listen_port: 8000, forward_host: "", forward_port: 3306, protocol: "tcp" });
      }
    });
  };

  const handleDeleteStream = (port: number) => {
    if (!confirm(t('streams.deleteConfirm', { port }))) return;
    deleteStreamMutation.mutate(port);
  };

  return (
    <Card>
       <CardHeader className="pb-3">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <CardTitle>{t('streams.title')}</CardTitle>
                <CardDescription>{t('streams.description')}</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="outline" size="icon" onClick={() => refetch()} disabled={isLoading}>
                  <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                </Button>
                {canManageHosts && (
                <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
                  <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" /> {t('streams.addStream')}</Button></DialogTrigger>
                  <DialogContent>
                    <DialogHeader><DialogTitle>{t('streams.addStream')}</DialogTitle></DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <Label>{t('streams.listenPort')}</Label>
                        <Input type="number" value={newStream.listen_port} onChange={e => setNewStream({...newStream, listen_port: parseInt(e.target.value)})} placeholder="8000" />
                      </div>
                      <div className="grid gap-2">
                         <Label>{t('streams.protocol')}</Label>
                         <Select value={newStream.protocol} onValueChange={v => setNewStream({...newStream, protocol: v as any})}>
                           <SelectTrigger><SelectValue /></SelectTrigger>
                           <SelectContent>
                             <SelectItem value="tcp">TCP</SelectItem>
                             <SelectItem value="udp">UDP</SelectItem>
                           </SelectContent>
                         </Select>
                      </div>
                      <div className="grid gap-2">
                        <Label>{t('streams.forwardHost')}</Label>
                        <Input value={newStream.forward_host} onChange={e => setNewStream({...newStream, forward_host: e.target.value})} placeholder="127.0.0.1" />
                      </div>
                      <div className="grid gap-2">
                        <Label>{t('streams.forwardPort')}</Label>
                        <Input type="number" value={newStream.forward_port} onChange={e => setNewStream({...newStream, forward_port: parseInt(e.target.value)})} placeholder="3306" />
                      </div>
                    </div>
                    <DialogFooter>
                       <Button onClick={handleAddStream} disabled={addStreamMutation.isPending}>
                           {addStreamMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                           {t('streams.save')}
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
                 <TableHead>{t('streams.listenPort')}</TableHead>
                 <TableHead>{t('streams.protocol')}</TableHead>
                 <TableHead>Destination</TableHead>
                 <TableHead className="text-right">{t('streams.actions')}</TableHead>
               </TableRow>
             </TableHeader>
             <TableBody>
               {streams?.map(stream => (
                 <TableRow key={stream.id}>
                    <TableCell className="font-mono">:{stream.listen_port}</TableCell>
                    <TableCell><Badge variant="outline">{stream.protocol.toUpperCase()}</Badge></TableCell>
                    <TableCell className="font-mono">{stream.forward_host}:{stream.forward_port}</TableCell>
                    <TableCell className="text-right">
                        {canManageHosts && (
                          <Button variant="ghost" size="sm" onClick={() => handleDeleteStream(stream.listen_port)}>
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        )}
                    </TableCell>
                 </TableRow>
               ))}
               {(!streams || streams.length === 0) && (
                   <TableRow>
                       <TableCell colSpan={4} className="text-center text-muted-foreground h-24">No active streams</TableCell>
                   </TableRow>
               )}
             </TableBody>
          </Table>
       </CardContent>
    </Card>
  );
}
