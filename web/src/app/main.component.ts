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
        <span *ngIf="reveal">: {{user.value.points}}</span>
        <span *ngIf="!reveal && user.value.points != null">: x</span>
      </li>
    </ul>
    <h2>Cards</h2>
    <div *ngIf="displayCards">
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
  reveal: boolean = false;
  displayCards: boolean = true;
  cards: number[] = [0, 1, 2, 3, 5, 8, 13, 20, 40, 60, 100];

  constructor(private sessionService: SessionService) {
    sessionService.session.subscribe((session: Session | null) => { this.session = session; });
    sessionService.state.subscribe((state: SessionState | null) => {
      this.state = state;
      if (this.state != null) {
        // Check whether to reveal cards
        let uids = Object.keys(this.state.users);
        this.reveal = true;
        uids.forEach((uid: string) => {
          let user = this.state?.users[uid];
          if (user !== undefined && user.points === null) {
            this.reveal = false;
          }
        });
        // Check wheter to display card buttons
        let ownUid = this.sessionService.uidValue() as string;
        if (ownUid !== null) {
          let ownUser = this.state?.users[ownUid] as UserState;
          this.displayCards = ownUser.points === null;
        }
      }
    });
  }

  setPoints(points: number) {
    this.sessionService.setPoints(points);
  }

  resetPoints() {
    this.sessionService.resetPoints();
  }
}
