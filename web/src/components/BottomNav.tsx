import { NavLink } from "react-router-dom";

const NAV = [
  { to: "/dashboard", icon: "⚡", label: "Dashboard" },
  { to: "/streams",   icon: "🔄", label: "Streams" },
  { to: "/contacts",  icon: "👥", label: "Contacts" },
  { to: "/settings",  icon: "⚙️",  label: "Settings" },
];

export default function BottomNav() {
  return (
    <nav style={{
      position: "fixed", bottom: 0, left: "50%", transform: "translateX(-50%)",
      width: "100%", maxWidth: "430px",
      background: "var(--surface)", borderTop: "1px solid var(--border)",
      display: "flex", height: "64px",
      paddingBottom: "env(safe-area-inset-bottom)",
    }}>
      {NAV.map(({ to, icon, label }) => (
        <NavLink
          key={to}
          to={to}
          style={({ isActive }) => ({
            flex: 1, display: "flex", flexDirection: "column",
            alignItems: "center", justifyContent: "center",
            textDecoration: "none", gap: "2px",
            color: isActive ? "var(--accent)" : "var(--text-muted)",
            fontSize: "0.65rem", fontWeight: isActive ? 700 : 400,
          })}
        >
          <span style={{ fontSize: "1.3rem" }}>{icon}</span>
          {label}
        </NavLink>
      ))}
    </nav>
  );
}
