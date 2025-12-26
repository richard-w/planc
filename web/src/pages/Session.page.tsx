import * as mc from "@mantine/core";
import * as react from "react";
import * as react_router_dom from "react-router-dom";
import { useSessionControl } from "../context/SessionControlProvider";
import Cards from "../components/Cards.component";
import AdminPanel from "../components/AdminPanel.component";
import UserTable from "../components/UserTable.component";
import Statistics from "../components/Statistics.component";

export function SessionPage() {
  const sessionControl = useSessionControl();
  const navigate = react_router_dom.useNavigate();
  react.useEffect(() => {
    if (sessionControl.sessionId === undefined) {
      navigate("/login");
    }
  }, [sessionControl.sessionId]);
  if (sessionControl.uid === undefined || sessionControl.sessionState === undefined) {
    return <mc.Loader />
  }
  const leaveSession = () => {
    sessionControl.resetSession();
  };
  const selfState = sessionControl.sessionState.users[sessionControl.uid];

  return (
    <>
    <h2>Session {sessionControl.sessionId}</h2>
    <h3>Users</h3>
    <UserTable />
    <h3>Cards</h3>
    <Cards visible={selfState.points === null && !selfState.isSpectator} />
    <h3>Statistics</h3>
    <Statistics />
    <h3>Controls</h3>
    <mc.Checkbox label="Spectator" onChange={(event) => sessionControl.setSpectator(event.currentTarget.checked)}/>
    <AdminPanel />
    <mc.Space h="xl" />
    <mc.Button onClick={() => leaveSession()}>Leave</mc.Button>
    </>
  )
}
