import * as mc_notifications from "@mantine/notifications";
import * as react from "react";

export interface SessionControl {
  readonly sessionId: string | undefined;
  readonly userName: string | undefined;
  readonly uid: string | undefined;
  readonly sessionState: SessionState | undefined;
  readonly isAdmin: boolean;
  readonly revealPoints: boolean;

  joinSession(userName: string, sessionId: string): void;
  resetSession(): void;
  setPoints(points: string): void;
  resetPoints(): void;
  claimSession(): void;
  kickUser(userId: string): void;
  setSpectator(isSpectator: boolean): void;
}

export interface Session {
  readonly userName: string,
  readonly sessionId: string,
}

export interface SessionState {
  readonly users: UserStateMap;
  readonly admin: string;
}

export interface UserStateMap {
  [key: string]: UserState;
}

export interface UserState {
  readonly name: string | undefined;
  readonly points: number | undefined;
  readonly isSpectator: boolean;
}

export interface SessionControlProviderProps {
  children: react.ReactNode
}

const SESSION_CONTROL_CONTEXT = react.createContext<SessionControl>({
  joinSession: function (): void {
    throw new Error("Function not implemented.");
  },
  resetSession: function (): void {
    throw new Error("Function not implemented.");
  },
  setPoints: function (): void {
    throw new Error("Function not implemented.");
  },
  resetPoints: function (): void {
    throw new Error("Function not implemented.");
  },
  claimSession: function (): void {
    throw new Error("Function not implemented.");
  },
  kickUser: function (): void {
    throw new Error("Function not implemented.");
  },
  setSpectator: function (): void {
    throw new Error("Function not implemented.");
  },
  sessionId: undefined,
  userName: undefined,
  uid: undefined,
  sessionState: undefined,
  isAdmin: false,
  revealPoints: false,
});

export default function SessionControlProvider({ children }: SessionControlProviderProps) {
  const [userName, setUserName] = react.useState<string | undefined>(undefined);
  const [uid, setUid] = react.useState<string | undefined>(undefined);
  const [sessionId, setSessionId] = react.useState<string | undefined>(undefined);
  const [webSocket, setWebSocket] = react.useState<WebSocket | undefined>(undefined);
  const [sessionState, setSessionState] = react.useState<SessionState | undefined>(undefined);
  const isAdmin = uid !== undefined && uid === sessionState?.admin;

  // Only reveal points if all users that are not spectators have chosen.
  let revealPoints = true;
  let numNonSpectators = 0;
  for (const uid in sessionState?.users) {
    const user = sessionState.users[uid];
    if (user.points === null && !user.isSpectator) {
      revealPoints = false;
      numNonSpectators += 1;
    }
  }
  if (numNonSpectators === 0) {
    revealPoints = false;
  }

  const sessionControl: SessionControl = {
    joinSession: (userName, sessionId) => {
      setUserName(userName);
      setSessionId(sessionId);
    },
    resetSession: () => {
      if (webSocket !== undefined) {
        webSocket.onclose = () => {};
        webSocket.close();
      }
      setUserName(undefined);
      setUid(undefined);
      setSessionId(undefined);
      setWebSocket(undefined);
      setUid(undefined);
      setSessionState(undefined);
    },
    setPoints: (points: string) => {
      webSocket?.send(JSON.stringify({ tag: "SetPoints", content: points }));
    },
    resetPoints: () => {
      webSocket?.send(JSON.stringify({ tag: "ResetPoints", content: null }));
    },
    claimSession: () => {
      webSocket?.send(JSON.stringify({ tag: "ClaimSession", content: null }));
    },
    kickUser: (userId: string) => {
      webSocket?.send(JSON.stringify({ tag: "KickUser", content: userId }));
    },
    setSpectator: (isSpectator: boolean) => {
      webSocket?.send(JSON.stringify({ tag: "SetSpectator", content: isSpectator }));
    },
    sessionId,
    userName,
    uid,
    sessionState,
    isAdmin,
    revealPoints,
  };
  react.useEffect(() => {
    if (sessionId === undefined) {
      return;
    }
    const ws = new WebSocket(webSocketUrl(sessionId));
    ws.onopen = (event) => {
      console.log("WebSocket opened: ", event);
      // Request the user id.
      ws.send(JSON.stringify({tag: "Whoami", content: null }));
      // Change the username. Also triggers a session broadcast.
      ws.send(JSON.stringify({tag: "NameChange", content: userName }));
      setWebSocket(ws);
    };
    ws.onerror = (event) => {
      mc_notifications.showNotification({
          message: "Session error: " + event,
      });
      console.log("WebSocket error: ", event);
      ws.onclose = () => {};
      ws.close();
      sessionControl.resetSession();
    };
    ws.onclose = (event) => {
      console.log("WebSocket closed: ", event);
      ws.onclose = () => {};
      ws.close();
      sessionControl.resetSession();
    };
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      switch (message.tag) {
        case "Error": {
          ws.onclose = () => {};
          ws.close();
          sessionControl.resetSession();
          mc_notifications.showNotification({
            message: "Error: " + message.content,
          });
          break;
        }
        case "Whoami": {
          setUid(message.content as string);
          break;
        }
        case "State": {
          setSessionState(message.content as SessionState);
          break;
        }
        case "KeepAlive": {
          break;
        }
        default: {
          throw new Error("Unexpected message tag: " + message.tag);
        }
      }
    };
    return () => {
      console.log("Closing WebSocket")
      ws.onclose = () => {};
      ws.close();
    };
  }, [sessionId]);
  return (
    <SESSION_CONTROL_CONTEXT.Provider value={sessionControl}>
      {children}
    </SESSION_CONTROL_CONTEXT.Provider>
  );
}

function webSocketUrl(sessionId: string): string {
  // Establish connection to session.
  let url: string = '';
  if (window.location.protocol === 'https:') {
    url += 'wss://';
  } else {
    url += 'ws://';
  }
  url += window.location.hostname
  if (window.location.port !== "") {
    url += ':' + window.location.port;
  }
  url += '/api/';
  url += sessionId;
  return url;
}

export function useSessionControl(): SessionControl {
  return react.useContext(SESSION_CONTROL_CONTEXT);
}
