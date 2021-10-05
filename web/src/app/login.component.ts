import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { MatDialog } from '@angular/material/dialog';
import { SessionService } from './session.service';

class LoginDialogResult {
  name: string = '';
  sessionId: string = '';
};

@Component({
  selector: 'app-login-dialog',
  template: `
    <div class="login-dialog-content" mat-dialog-content>
      <mat-form-field appearance="fill">
        <mat-label>Your Name</mat-label>
        <input type="text" name="name" matInput [(ngModel)]="result.name" />
      </mat-form-field>
      <br />
      <mat-form-field appearance="fill">
        <mat-label>Session ID</mat-label>
        <input type="text" name="name" matInput [(ngModel)]="result.sessionId" />
      </mat-form-field>
    </div>
    <div mat-dialog-actions>
      <button mat-raised-button color="primary" [mat-dialog-close]="result" [disabled]="!isFormComplete()">Go</button>
    </div>
  `,
  styles: [
    '.login-dialog-content { overflow: visible; }',
  ]
})
export class LoginDialogComponent {
  result: LoginDialogResult = new LoginDialogResult();

  isFormComplete() {
    if (this.result.name == '') return false;
    if (this.result.sessionId == '') return false;
    return true;
  }
}

@Component({
  selector: 'app-login',
  template: '',
  styles: []
})
export class LoginComponent implements OnInit {
  constructor(
    private dialog: MatDialog,
    private router: Router,
    private sessionService: SessionService,
  ) {}

  ngOnInit() {
    this.openLoginDialog();
  }

  openLoginDialog() {
    const dialogRef = this.dialog.open(LoginDialogComponent, {
      closeOnNavigation: false,
      disableClose: true,
    });
    dialogRef.afterClosed().subscribe(result => {
      this.sessionService
        .joinSession(result.sessionId, result.name)
        .then(() => this.router.navigate(['/']))
        .catch(err => {
          alert(err);
          this.openLoginDialog();
        });
    });
  }
}
