import { ReactNode, useState } from 'react';
import { Sidebar, type SidebarItem } from './Sidebar.tsx';
import { Header } from './Header.tsx';

interface AppShellProps {
  sidebarItems: SidebarItem[];
  children: ReactNode;
  headerActions?: ReactNode;
  sidebarFooter?: ReactNode;
  phaseLabel?: string;
  phaseStatus?: ReactNode;
}

export function AppShell({
  sidebarItems,
  children,
  headerActions,
  sidebarFooter,
  phaseLabel,
  phaseStatus,
}: AppShellProps) {
  const [mobileNavOpen, setMobileNavOpen] = useState(false);

  return (
    <div className="flex min-h-screen bg-background text-foreground">
      <aside className="hidden w-64 border-r border-border/60 bg-background lg:flex">
        <Sidebar items={sidebarItems} footer={sidebarFooter} />
      </aside>

      <div className="relative flex flex-1 flex-col">
        <Header
          onOpenMobileNav={() => setMobileNavOpen(true)}
          actions={headerActions}
          phaseLabel={phaseLabel}
          phaseStatus={phaseStatus}
        />
        <div className="flex-1 bg-muted/10">
          <div className="mx-auto h-full w-full max-w-6xl px-4 py-6 sm:px-6">{children}</div>
        </div>
      </div>

      {mobileNavOpen ? (
        <div className="lg:hidden">
          <div
            role="presentation"
            className="fixed inset-0 z-40 bg-background/40 backdrop-blur-sm"
            onClick={() => setMobileNavOpen(false)}
          />
          <div className="fixed inset-y-0 left-0 z-50 w-72 border-r border-border/60 bg-background shadow-xl">
            <Sidebar
              items={sidebarItems}
              footer={sidebarFooter}
              onNavigate={() => setMobileNavOpen(false)}
              onClose={() => setMobileNavOpen(false)}
            />
          </div>
        </div>
      ) : null}
    </div>
  );
}
