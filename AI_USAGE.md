# AI Tool Usage

- Windsurf for api boilerplate and writign migrations. Shape and index for tables was decided by me
- Cursor to understand state machine's states such as draft/void/uncollectible and to validate my state design pattern and its trigger


# Decisions made by me

- While storing invoice and invoice items in db, I made these two db ops coupled into one txn.
- Assuming that the PSP would also store payment_attmepts in their own db, and integarating that into my invoice/{id}/flow
- Reconcile flow on invocie service to regularly check the status of PENDING payemnt_attempts by fetching status from PSP.

# AI's correctness
- While thinking for solution for concurrent payemnts on same invoice, i was hesitant on using row locking as the lock was acquired till full api call to PSP. Although i have maintained a timeout of 7 secs. But the tradeoff of 7s compared to optimistic locking was very small. So it helped me in deciding row locking 