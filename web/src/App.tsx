import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import * as mc from "@mantine/core";
import * as mc_notifications from "@mantine/notifications";
import { theme } from "./theme.ts";
import { Router } from "./Router.tsx";
import SessionControlProvider from "./context/SessionControlProvider.tsx";

export default function App() {
  return (
    <mc.MantineProvider theme={theme}>
      <mc_notifications.Notifications />
      <SessionControlProvider>
        <AppInner />
      </SessionControlProvider>
    </mc.MantineProvider>
  );
}

function AppInner() {
  const colors = theme.colors![theme.primaryColor!]!;
  return (
    <mc.AppShell
      header={{
        height: 90,
      }}
    >
      <mc.AppShell.Header
        style={{
          backgroundColor: colors[7],
        }}
      >
        <h1 style={{ color: "white" }}>Planc</h1>
      </mc.AppShell.Header>
      <mc.AppShell.Main>
        <Router />
      </mc.AppShell.Main>
    </mc.AppShell>
  );
}
