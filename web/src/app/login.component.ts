import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { MatDialog } from '@angular/material/dialog';
import { SessionService } from './session.service';

class LoginDialogResult {
  name: string = '';
  joinToken: string = '';
  createSession: boolean = false;
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
        <input type="text" name="name" matInput [(ngModel)]="result.joinToken" [disabled]="result.createSession"/>
      </mat-form-field>
      <br />
      <mat-checkbox [(ngModel)]="result.createSession" (change)="onCreateSessionChange()">Create Session</mat-checkbox>
    </div>
    <div mat-dialog-actions>
      <button mat-raised-button color="primary" mat-dialog-close [disabled]="!isFormComplete()">Go</button>
    </div>
  `,
  styles: [
    '.login-dialog-content { overflow: visible; }',
  ]
})
export class LoginDialogComponent {
  result: LoginDialogResult = new LoginDialogResult();

  onCreateSessionChange() {
    this.result.joinToken = '';
  }

  isFormComplete() {
    if (this.result.name == '') return false;
    if (this.result.joinToken == '' && !this.result.createSession) return false;
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
    const dialogRef = this.dialog.open(LoginDialogComponent, {
      closeOnNavigation: false,
      disableClose: true,
    });
    dialogRef.afterClosed().subscribe(result => {
      var callback = (success: boolean, message: string | null) => {
        if (success) {
          this.router.navigate(['/']);
        } else {
          alert(message);
        }
      };
      if (result.createSession) {
        this.sessionService.createSession(result.name, callback);
      } else {
        this.sessionService.joinSession(result.name, result.joinToken, callback);
      }
    });
  }
}
