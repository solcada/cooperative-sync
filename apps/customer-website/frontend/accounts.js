const paymentForm = document.getElementById("payment-form");
const formStatus = document.getElementById("form-status");

paymentForm.addEventListener("submit", (event) => {
  event.preventDefault();
  formStatus.textContent = "Payment information updated.";
});
