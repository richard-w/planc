import { Injectable } from '@angular/core';
import { Router } from '@angular/router';
import { Subject, BehaviorSubject, Observable } from 'rxjs';
import { webSocket, WebSocketSubject } from 'rxjs/webSocket';

export class Session {
  name: string = '';
  sessionId: string = '';
}

export class SessionState {
  users: UserStateMap = {};
}

export interface UserStateMap {
  [key: string]: UserState;
}

export class UserState {
  name: string | null = null;
  points: number | null = null;
}

class Message {
  tag: string = '';
  content: any = null;
}

@Injectable({
  providedIn: 'root'
})
export class SessionService {
  private webSocket: WebSocketSubject<Message> | null = null;
  private sessionSubject: BehaviorSubject<Session | null>;
  private stateSubject: BehaviorSubject<SessionState | null>;
  private uidSubject: BehaviorSubject<string | null>;

  public session: Observable<Session | null>;
  public state: Observable<SessionState | null>;
  public uid: Observable<string | null>;

  constructor(
    private router: Router,
  ) {
    this.sessionSubject = new BehaviorSubject<Session | null>(null);
    this.session = this.sessionSubject.asObservable();
    this.stateSubject = new BehaviorSubject<SessionState | null>(null);
    this.state = this.stateSubject.asObservable();
    this.uidSubject = new BehaviorSubject<string | null>(null);
    this.uid = this.uidSubject.asObservable();
  }

  joinSession(name: string, sessionId: string) {
    // Establish connection to session.
    var webSocketUrl: string = '';
    if (window.location.protocol === 'https:') {
      webSocketUrl += 'wss://';
    } else {
      webSocketUrl += 'ws://';
    }
    webSocketUrl += window.location.hostname
    if (window.location.port !== "") {
      webSocketUrl += ':' + window.location.port;
    }
    webSocketUrl += '/api/';
    webSocketUrl += sessionId;
    
    this.webSocket = webSocket({
      url: webSocketUrl,
      openObserver: {
        next: () => {
          this.sessionSubject.next({
            name: name,
            sessionId: sessionId,
          });
          this.router.navigate(['/']);
        },
      },
      closeObserver: {
        next: (closeEvent) => {
          this.leaveSession();
        },
      },
    });
    // Subscribe to server messages.
    this.webSocket.subscribe(msg => this.handleServerMessage(msg));

    // Send the name change message to initialize the connection.
    this.webSocket.next({tag: "Whoami", content: null });
    this.webSocket.next({tag: "NameChange", content: name });
  }

  leaveSession() {
    if (this.webSocket != null) {
      this.webSocket.complete();
      this.webSocket = null;
    }
    this.sessionSubject.next(null);
    this.stateSubject.next(null);
    this.router.navigate(['/login']);
  }

  setPoints(points: number) {
    this.webSocket?.next({ tag: "SetPoints", content: points });
  }

  resetPoints() {
    this.webSocket?.next({ tag: "ResetPoints", content: null });
  }

  handleServerMessage(msg: Message) {
    switch (msg.tag) {
      case "State": {
        console.log("Received 'State' Message" + JSON.stringify(msg.content));
        this.stateSubject.next(msg.content as SessionState);
        break;
      }
      case "Whoami": {
        console.log("Received 'Whoami' Message" + JSON.stringify(msg.content));
        this.uidSubject.next(msg.content as string);
        break;
      }
      default: {
        console.log("Undefined message: " + msg.content);
        break;
      }
    }
  }

  sessionValue(): Session | null {
    return this.sessionSubject.value;
  }

  stateValue(): SessionState | null {
    return this.stateSubject.value;
  }

  uidValue(): string | null {
    return this.uidSubject.value;
  }

}
