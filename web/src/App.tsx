import { Routes, Route, Navigate } from "react-router-dom";
import OnboardingPage from "./components/OnboardingPage.tsx";
import DashboardPage from "./components/DashboardPage.tsx";
import StreamsPage from "./components/StreamsPage.tsx";
import ContactsPage from "./components/ContactsPage.tsx";
import SettingsPage from "./components/SettingsPage.tsx";
import BottomNav from "./components/BottomNav.tsx";
import { useIdentityStore } from "./store/identityStore.ts";

export default function App() {
  const npub = useIdentityStore((s) => s.npub);

  if (!npub) {
    return <OnboardingPage />;
  }

  return (
    <div className="app">
      <main className="app__content">
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/streams" element={<StreamsPage />} />
          <Route path="/contacts" element={<ContactsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </main>
      <BottomNav />
    </div>
  );
}
