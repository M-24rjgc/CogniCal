import AnalyticsOverview from '../components/analytics/AnalyticsOverview';
import { WorkloadForecastBanner } from '../components/analytics/WorkloadForecastBanner';
import { WeeklySummaryPanel } from '../components/wellness/WeeklySummaryPanel';
import { WellnessNudgeToast } from '../components/wellness/WellnessNudgeToast';
import { useLatestWorkloadForecasts } from '../hooks/useWorkloadForecast';
import { usePendingNudge } from '../hooks/useWellness';

export default function DashboardPage() {
  const { data: forecasts } = useLatestWorkloadForecasts();
  const { data: pendingNudge } = usePendingNudge();

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6 p-6 pb-14">
        {forecasts && forecasts.length > 0 && <WorkloadForecastBanner forecasts={forecasts} />}

        {/* 分析仪表盘 - 全宽显示 */}
        <AnalyticsOverview />

        {/* 每周健康概览 - 放在下方 */}
        <WeeklySummaryPanel />
      </div>
      <WellnessNudgeToast nudge={pendingNudge ?? null} />
    </div>
  );
}
