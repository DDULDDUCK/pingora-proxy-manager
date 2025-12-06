import { Activity, HardDrive, ShieldCheck, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { useRealtimeStats, useHistoryStats } from "@/hooks/useStats";

export function DashboardTab() {
  const { t } = useTranslation();
  const { data: realtime } = useRealtimeStats();
  const { data: history } = useHistoryStats();

  const formatBytes = (bytes: number) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{t('dashboard.totalRequests')}</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{realtime?.requests.toLocaleString() ?? "-"}</div>
            <p className="text-xs text-muted-foreground">
              {t('dashboard.sinceRestart')}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{t('dashboard.dataTransfer')}</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatBytes(realtime?.bytes ?? 0)}</div>
            <p className="text-xs text-muted-foreground">
              {t('dashboard.totalBandwidth')}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{t('dashboard.successRate')}</CardTitle>
            <ShieldCheck className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{realtime?.status_2xx.toLocaleString() ?? "-"}</div>
            <p className="text-xs text-muted-foreground">
              {t('dashboard.successfulResponses')}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">{t('dashboard.errors')}</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {((realtime?.status_4xx ?? 0) + (realtime?.status_5xx ?? 0)).toLocaleString()}
            </div>
            <p className="text-xs text-muted-foreground">
              {t('dashboard.clientServerErrors')}
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className="col-span-4">
        <CardHeader>
          <CardTitle>{t('dashboard.trafficOverview')}</CardTitle>
          <CardDescription>{t('dashboard.requestsPerMinute')}</CardDescription>
        </CardHeader>
        <CardContent className="pl-2">
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={history ?? []}>
                <defs>
                  <linearGradient id="colorReqs" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8}/>
                    <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <XAxis dataKey="time" fontSize={12} tickLine={false} axisLine={false} />
                <YAxis fontSize={12} tickLine={false} axisLine={false} />
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <Tooltip />
                <Area type="monotone" dataKey="total_requests" stroke="#8884d8" fillOpacity={1} fill="url(#colorReqs)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
