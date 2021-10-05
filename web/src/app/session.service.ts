import { Injectable } from '@angular/core';
import { Subject, BehaviorSubject, Observable } from 'rxjs';
import { takeWhile } from 'rxjs/operators';
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
  public session$: Observable<Session | null>;
  public session(): Session | null { return this.sessionSubject.value; }

  constructor() {
    this.sessionSubject = new BehaviorSubject<Session | null>(null);
    this.session$ = this.sessionSubject.asObservable();
  }

  private static webSocketUrl(sessionId: string): string {
    // Establish connection to session.
    var url: string = '';
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

  public async joinSession(sessionId: string, name: string): Promise<void> {
    if (this.session() != null) {
      throw new Error("Already joined to a session");
    }

    // Open websocket
    this.webSocket = webSocket({
        url: SessionService.webSocketUrl(sessionId),
    });
    // Create new subject and use it to broadcast websocket messages.
    // Necessary because WebSocketSubject closes and reopens the connection
    // on the second subsecribe apparently.
    let messageSubject = new Subject<Message>();
    this.webSocket.subscribe(
      (msg) => messageSubject.next(msg),
      (err) => messageSubject.next(err),
      () => messageSubject.complete(),
    );

    // Request the user id.
    this.webSocket.next({tag: "Whoami", content: null });
    // Change the username. Also triggers a session broadcast.
    this.webSocket.next({tag: "NameChange", content: name });

    // Receive session information.
    var uid: string | null = null;
    var state: SessionState | null = null;
    await messageSubject
      .pipe(takeWhile((message) => {
        console.log("Got initial message: " + JSON.stringify(message));
        switch (message.tag) {
          case "Error": {
            this.webSocket?.complete();
            throw new Error(message.content);
            return false;
          }
          case "Whoami": {
            uid = message.content as string;
            break;
          }
          case "State": {
            state = message.content as SessionState;
            break;
          }
          default: {
            throw new Error("Unexpected message tag: " + message.tag);
          }
        }
        return uid === null || state === null;
      }))
      .toPromise();

    // Update webSocket and session members.
    this.sessionSubject.next({
      name: name,
      sessionId: sessionId,
      uid: uid!,
      state: state!,
    });

    // React to state changes and errors.
    messageSubject.subscribe(
      (message) => {
        console.log("Got update: " + JSON.stringify(message));
        switch (message.tag) {
          case "Error": {
            this.webSocket?.complete();
            this.sessionSubject.error(new Error(message.content));
            break;
          }
          case "State": {
            let session = this.session();
            if (session !== null) {
              session.state = message.content as SessionState;
              this.sessionSubject.next(session);
            }
            break;
          }
          default: {
            console.log("Unexpected message tag: " + JSON.stringify(message));
            break;
          }
        }
      },
      (err) => {
        console.log("Got update error: " + err);
        this.webSocket?.complete();
        this.sessionSubject.error(err);
      },
      () => {
        console.log("Updates completed");
      }
    );
  }

  public leaveSession() {
    this.webSocket?.complete();
    this.sessionSubject.next(null);
  }

  public setPoints(points: number) {
    this.webSocket?.next({ tag: "SetPoints", content: points });
  }

  resetPoints() {
    this.webSocket?.next({ tag: "ResetPoints", content: null });
  }

  claimSession() {
    this.webSocket?.next({ tag: "ClaimSession", content: null });
  }
}
