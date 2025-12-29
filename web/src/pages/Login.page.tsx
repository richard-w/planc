import * as react from "react";
import * as react_router_dom from "react-router-dom";
import * as mc from "@mantine/core";
import { useSessionControl } from "../context/SessionControlProvider";

const STORAGE_USERNAME_KEY = "login_userName";
const STORAGE_SESSIONID_KEY = "login_sessionId";
const STORAGE_REMEMBERME_KEY = "login_rememberMe";

export function LoginPage() {
  const [userName, setUserName] = react.useState(localStorage.getItem(STORAGE_USERNAME_KEY) ?? "");
  const [sessionId, setSessionId] = react.useState(localStorage.getItem(STORAGE_SESSIONID_KEY) ?? "");
  const [rememberMe, setRememberMe] = react.useState(localStorage.getItem(STORAGE_REMEMBERME_KEY) === "true");
  const sessionControl = useSessionControl();
  const navigate = react_router_dom.useNavigate();
  const onSubmit = () => {
    sessionControl.joinSession(userName, sessionId);
    if (rememberMe) {
      localStorage.setItem(STORAGE_USERNAME_KEY, userName);
      localStorage.setItem(STORAGE_SESSIONID_KEY, sessionId);
      localStorage.setItem(STORAGE_REMEMBERME_KEY, "true");
    } else {
      localStorage.removeItem(STORAGE_USERNAME_KEY);
      localStorage.removeItem(STORAGE_SESSIONID_KEY);
      localStorage.removeItem(STORAGE_REMEMBERME_KEY);
      setUserName("");
      setSessionId("");
    }
    navigate("/");
  };
  return (
    <mc.Container strategy="grid">
      <mc.Box>
        <mc.Stack gap="xs">
          <mc.TextInput label="Your Name" value={userName} onChange={(event) => setUserName(event.currentTarget.value)} />
          <mc.TextInput label="Session ID" value={sessionId} onChange={(event) => setSessionId(event.currentTarget.value)} />
          <mc.Checkbox label="Remember me" checked={rememberMe} onChange={(event) => setRememberMe(event.currentTarget.checked)} />
          <mc.Button onClick={onSubmit}>Go</mc.Button>
        </mc.Stack>
      </mc.Box>
    </mc.Container>
  );
}
