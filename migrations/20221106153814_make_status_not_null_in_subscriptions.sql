BEGIN;
  UPDATE subscription
    SET status = 'confirmed'
    WHERE status IS NULL;

    ALTER TABLE subscription
      ALTER COLUMN status SET NOT NULL;
COMMIT;
