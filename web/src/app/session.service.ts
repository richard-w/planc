import { Injectable } from '@angular/core';
import { Router } from '@angular/router';
import { Subject, BehaviorSubject, Observable } from 'rxjs';
import { webSocket, WebSocketSubject } from 'rxjs/webSocket';

export class Session {
  name!: string;
  sessionId!: string;
  uid!: string;
  state!: SessionState;
}

export class SessionState {
  users!: UserStateMap;
  admin!: string;
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
  public session: Observable<Session | null>;

  private name: string = '';
  private sessionId: string = '';
  private uid: string | null = null;
  private state: SessionState | null = null;

  constructor(
    private router: Router,
  ) {
    this.sessionSubject = new BehaviorSubject<Session | null>(null);
    this.session = this.sessionSubject.asObservable();
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
      closeObserver: {
        next: (closeEvent) => {
          this.leaveSession();
        },
      },
    });
    // Subscribe to server messages.
    this.webSocket.subscribe(msg => this.handleServerMessage(msg));

    // Request the user id.
    this.webSocket.next({tag: "Whoami", content: null });
    // Send the name change message to initialize the connection.
    this.webSocket.next({tag: "NameChange", content: name });
  }

  leaveSession() {
    if (this.webSocket != null) {
      this.webSocket.complete();
      this.webSocket = null;
    }
    this.sessionSubject.next(null);
    this.name = '';
    this.sessionId = '';
    this.uid = null;
    this.state = null;
    this.router.navigate(['/login']);
  }

  setPoints(points: number) {
    this.webSocket?.next({ tag: "SetPoints", content: points });
  }

  resetPoints() {
    this.webSocket?.next({ tag: "ResetPoints", content: null });
  }

  claimSession() {
    this.webSocket?.next({ tag: "ClaimSession", content: null });
  }

  handleServerMessage(msg: Message) {
    switch (msg.tag) {
      case "State": {
        console.log("Received 'State' Message" + JSON.stringify(msg.content));
        this.state = msg.content as SessionState;
        break;
      }
      case "Whoami": {
        console.log("Received 'Whoami' Message" + JSON.stringify(msg.content));
        this.uid = msg.content as string;
        break;
      }
      case "Error": {
        console.log("Received 'Error' Message" + JSON.stringify(msg.content));
        alert("Error: " + msg.content);
        this.leaveSession();
        break;
      }
      default: {
        console.log("Undefined message: " + msg.content);
        break;
      }
    }
    if (this.uid !== null && this.state !== null) {
      console.log("Updating session");
      this.sessionSubject.next({
        name: this.name,
        sessionId: this.sessionId,
        uid: this.uid,
        state: this.state,
      });
      this.router.navigate(['/']);
    }
  }

  sessionValue(): Session | null {
    return this.sessionSubject.value;
  }
}
