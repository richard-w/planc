import * as mc from "@mantine/core";
import * as tablerIcons from "@tabler/icons-react";
import { useSessionControl } from "../context/SessionControlProvider";

export interface AdminUserActionsProps {
  readonly uid: string;
}

export default function AdminUserActions(props: AdminUserActionsProps) {
  const sessionControl = useSessionControl();
  const adminActions = [];
  if (props.uid !== sessionControl.uid) {
    adminActions.push(<mc.ActionIcon key="kick" onClick={() => sessionControl.kickUser(props.uid)}><tablerIcons.IconUserMinus /></mc.ActionIcon>);
  }
  return (
    <mc.ActionIconGroup>
      {adminActions}
    </mc.ActionIconGroup>
  );
}
