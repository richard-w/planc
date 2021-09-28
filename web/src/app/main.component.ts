import { Component } from '@angular/core';
import { SessionService, Session, SessionState, UserState } from './session.service';

@Component({
  selector: 'app-main',
  template: `
    <h1>Session {{session?.sessionId}}</h1>
    <h2>Users</h2>
    <ul>
      <li *ngFor="let user of state?.users | keyvalue">
        {{user.value.name}}
        <span *ngIf="revealCards()">: {{user.value.points}}</span>
        <span *ngIf="!revealCards() && user.value.points != null">: x</span>
      </li>
    </ul>
    <h2>Cards</h2>
    <div *ngIf="displayCards()">
      <button mat-raised-button color="primary" *ngFor="let card of cards" (click)="setPoints(card)">{{card}}</button>
    </div>
    <h2>Control</h2>
    <button mat-raised-button color="primary" (click)="resetPoints()">Reset</button>
  `,
  styles: []
})
export class MainComponent {
  session: Session | null = null;
  state: SessionState | null = null;
  cards: number[] = [0, 1, 2, 3, 5, 8, 13, 20, 40, 60, 100];

  constructor(private sessionService: SessionService) {
    sessionService.session.subscribe((session: Session | null) => { this.session = session; });
    sessionService.state.subscribe((state: SessionState | null) => { this.state = state; });
  }

  setPoints(points: number) {
    this.sessionService.setPoints(points);
  }

  resetPoints() {
    this.sessionService.resetPoints();
  }

  revealCards(): boolean {
    if (this.state === null) return false;
    var reveal = true;
    Object.values(this.state.users).forEach((user) => {
      if (user.points === null) {
        reveal = false;
      }
    });
    return reveal;
  }

  displayCards(): boolean | null {
    if (this.state === null) return false;
    let ownUid = this.sessionService.uidValue();
    if (ownUid === null) return false;
    let ownUser = this.state?.users[ownUid]!;
    return ownUser.points === null;
  }
}
