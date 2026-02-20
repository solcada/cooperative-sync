# Feature: Website Sync Details

## Summary
Provide a basic customer-facing website that shows backup status and account billing inputs.

## User Needs
- See when the last successful sync happened.
- See how much data has been backed up.
- See how many files are currently backed up.
- Update payment details from an account page.

## v1 Scope
- Sync details page with:
  - Last successful sync timestamp.
  - Total backed up data amount.
  - Total file count.
- Account page with:
  - Cardholder name.
  - Card number.
  - Expiry date.
  - CVC.
  - Billing ZIP/postal code.
  - Save action with success feedback.

## Out of Scope
- Real payment processor integration.
- Real auth/session handling.
- Real backend API wiring.
- Full billing history/invoices.

## Acceptance Criteria
- Website has a page that displays the 3 requested sync metrics.
- Website has an account page where a user can submit updated payment data.
- Navigation between sync details and account page is available.
- UI works on desktop and mobile viewports.
