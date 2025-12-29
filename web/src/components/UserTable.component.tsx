import * as mc from "@mantine/core";
import * as react from "react";
import { useSessionControl } from "../context/SessionControlProvider";
import AdminUserActions from "./AdminUserActions";

const CHECK_MARK = "✅";
const CROSS_MARK = "❌";
const EYE_MARK = "👁";

export default function UserTable() {
  const sessionControl = useSessionControl();
  const sessionState = sessionControl.sessionState;
  if (sessionState === undefined) {
    return <></>;
  }

  // Setup table head row.
  const tableHead = [
    <mc.Table.Th key="user">User</mc.Table.Th>,
    <mc.Table.Th key="voted">Voted</mc.Table.Th>,
    <mc.Table.Th key="points">Points</mc.Table.Th>,
  ];
  if (sessionControl.isAdmin) {
    tableHead.push(<mc.Table.Th key="admin">Admin</mc.Table.Th>)
  }

  // Setup user rows.
  const rows: react.ReactNode[] = [];
  for (const uid in sessionState.users) {
    const user = sessionState.users[uid];
    const points = sessionControl.revealPoints ? user.points : "?";
    let voted;
    if (user.isSpectator) {
      voted = EYE_MARK;
    } else if (user.points !== null) {
      voted = CHECK_MARK;
    } else {
      voted = CROSS_MARK;
    }
    const cells = [
      <mc.Table.Td key="user">{user.name}</mc.Table.Td>,
      <mc.Table.Td key="voted">{voted}</mc.Table.Td>,
      <mc.Table.Td key="points">{points}</mc.Table.Td>,
    ];
    if (sessionControl.isAdmin) {
      cells.push(
        <mc.Table.Td key="admin">
          <AdminUserActions uid={uid} />
        </mc.Table.Td>
      );
    }
    rows.push(
      <mc.Table.Tr key={uid}>
        {cells}
      </mc.Table.Tr>
    );
  }

  return (
    <mc.Table verticalSpacing="xs">
      <mc.Table.Thead>
        <mc.Table.Tr>
          {tableHead}
        </mc.Table.Tr>
      </mc.Table.Thead>
      <mc.Table.Tbody>
        {rows}
      </mc.Table.Tbody>
    </mc.Table>
  );
}
