# session-payment pallet

A pallet that execute payments if user has given his consent.

Workflow:
  - user gives his consent to initiate payment from a giben IBAN and BIC CODE.
  - user execute a charging session.
  - when charging session is finished, a tariff smart contract calculate the price of the charging session and 
    initiate a payment for the user. 

