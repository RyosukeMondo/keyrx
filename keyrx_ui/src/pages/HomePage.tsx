import React, { useMemo } from 'react';
import { ActiveProfileCard } from '../components/ActiveProfileCard';
import { DeviceListCard } from '../components/DeviceListCard';
import { QuickStatsCard } from '../components/QuickStatsCard';
import { useProfiles } from '../hooks/useProfiles';

/**
 * HomePage / Dashboard
 *
 * Main dashboard view showing:
 * - Active profile card with quick access to edit
 * - Connected devices list with status
 * - Quick statistics (latency, events, uptime)
 *
 * Layout: From design.md Layout 1
 * Responsive: Cards stack vertically on mobile (<768px)
 */
export const HomePage: React.FC = () => {
  const { data: profiles, isLoading: profilesLoading } = useProfiles();

  // Find active profile and format for card
  const activeProfileData = useMemo(() => {
    const active = profiles?.find((p) => p.isActive);
    if (!active) return undefined;

    return {
      name: active.name,
      layers: 1, // TODO: Get from profile metadata when available
      mappings: active.keyCount || 0,
      modifiedAt: new Date(active.modifiedAt).toLocaleDateString(),
    };
  }, [profiles]);

  return (
    <main
      className="flex flex-col gap-4 md:gap-6 p-4 md:p-6 lg:p-8"
      role="main"
      aria-label="Dashboard"
    >
      <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold text-slate-100">
        Dashboard
      </h1>

      {/* Cards stack vertically on mobile, can be side-by-side on larger screens */}
      <div
        className="flex flex-col gap-4 md:gap-6"
        role="region"
        aria-label="Dashboard overview"
      >
        {/* Active Profile Card */}
        <ActiveProfileCard
          profile={activeProfileData}
          loading={profilesLoading}
        />

        {/* Connected Devices Card */}
        <DeviceListCard />

        {/* Quick Stats Card */}
        <QuickStatsCard />
      </div>
    </main>
  );
};

export default HomePage;
