import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { SessionService } from './session.service';

@Component({
  selector: 'app-root',
  template: `
    <mat-toolbar class="app-toolbar" color="primary">
      <span>planc</span>
      <span class="app-toolbar-spacer"></span>
      <button mat-icon-button *ngIf="isLoggedIn" (click)="onLogoutClick()">
        <mat-icon>logout</mat-icon>
      </button>
    </mat-toolbar>
    <router-outlet></router-outlet>
  `,
  styles: [
    '.app-toolbar { position: sticky; top: 0; z-index: 1; }',
    '.app-toolbar-spacer { flex: 1 1 auto; }',
  ]
})
export class AppComponent {

  isLoggedIn: boolean = false;

  constructor(
    private router: Router,
    private sessionService: SessionService,
  ) {
    this.sessionService.session$.subscribe((session) => {
      this.isLoggedIn = session != null;
    });
  }

  onLogoutClick() {
    this.sessionService.leaveSession();
    this.router.navigate(['/login']);
  }
}
