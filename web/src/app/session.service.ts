import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';

export class Session {
  name: string = '';
  token: string = '';
};

@Injectable({
  providedIn: 'root'
})
export class SessionService {
  private sessionSubject: BehaviorSubject<Session | null>;
  public session: Observable<Session | null>;

  constructor() {
    this.sessionSubject = new BehaviorSubject<Session | null>(null);
    this.session = this.sessionSubject.asObservable();
  }

  createSession(name: string, callback: (success: boolean, message: string | null) => void) {
    this.sessionSubject.next({
      name: name,
      token: 'abcd',
    });
    callback(true, null);
  }

  joinSession(name: string, joinToken: string, callback: (success: boolean, message: string | null) => void) {
    this.sessionSubject.next({
      name: name,
      token: joinToken,
    });
    callback(true, null);
  }

  leaveSession() {
    this.sessionSubject.next(null);
  }

  sessionValue(): Session | null {
    return this.sessionSubject.value;
  }
}
