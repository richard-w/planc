import * as mc from "@mantine/core";
import { useSessionControl } from "../context/SessionControlProvider";

export default function AdminPanel() {
  const sessionControl = useSessionControl();

  if (sessionControl.isAdmin) {
    return (
      <>
        <mc.Space h="xl" />
        <mc.Button onClick={() => sessionControl.resetPoints()}>Reset Points</mc.Button>
      </>
    );
  } else if (sessionControl.sessionState?.admin === null) {
    return (
      <>
        <mc.Space h="xl" />
        <mc.Button onClick={() => sessionControl.claimSession()}>Claim Session</mc.Button>
      </>
    );
  } else {
    return (
      <>
      </>
    );
  }
}
