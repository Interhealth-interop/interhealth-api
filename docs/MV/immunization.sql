CREATE OR REPLACE VIEW "DBAMV"."IMMUNIZATION"
(
    "immunization_code",
    "immunization_patient_code",
    "immunization_date",
    "immunization_time",
    "immunization_name",
    "immunization_dosage",
    "immunization_updated_date",
    "immunization_reason",
    "immunization_responsible"
) AS
SELECT DV.CD_VACINA AS "immunization_code",
       VDP.CD_PACIENTE AS "immunization_patient_code",
       to_char(VDP.DT_VACINA, 'YYYY-MM-DD') AS "immunization_date",
       to_char(VDP.DT_VACINA, 'HH24:MI:SS') AS "immunization_time",
       V.DS_VACINA AS "immunization_name",
       D.NM_DOSE AS "immunization_dosage",
       NULL AS "immunization_updated_date",
       NULL AS "immunization_reason",
       NULL AS "immunization_responsible"
  FROM DBAMV.PW_VACINA_DOSE_PACIENTE VDP
  LEFT JOIN DBAMV.PW_DOSE_VACINA DV ON VDP.CD_DOSE_VACINA = DV.CD_DOSE_VACINA
 INNER JOIN DBAMV.VACINA V ON DV.CD_VACINA = V.CD_VACINA
 INNER JOIN DBAMV.PW_DOSE D ON DV.CD_DOSE = D.CD_DOSE;
 