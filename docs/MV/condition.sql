CREATE OR REPLACE VIEW "DBAMV"."CONDITION"
(
    "condition_code",
    "condition_encounter_code",
    "condition_patient_code",
    "condition_name",
    "condition_classification", 
    "condition_type",
    "condition_provider_code",
    "condition_date",
    "condition_time",
    "condition_updated_date",
    "condition_reason",
    "condition_responsible"
) AS 
SELECT DA.CD_CID AS "condition_code",
       DA.CD_ATENDIMENTO AS "condition_encounter_code",
       A.CD_PACIENTE AS "condition_patient_code",
       CID.DS_CID AS "condition_name",
       NULL "condition_classification", 
       'CID'  "condition_type",
       PW.CD_PRESTADOR AS "condition_provider_code",
       TO_CHAR(DA.DH_DIAGNOSTICO,'YYYY-MM-DD') AS "condition_date",  
       TO_CHAR(DA.DH_DIAGNOSTICO,'HH24:MI:SS') AS "condition_time",
       NULL AS "condition_updated_date",
       NULL AS "condition_reason",
       NULL AS "condition_responsible"  
  FROM DBAMV.DIAGNOSTICO_ATENDIME DA
  LEFT JOIN DBAMV.ATENDIME A ON DA.CD_ATENDIMENTO = A.CD_ATENDIMENTO
  LEFT JOIN PW_DOCUMENTO_CLINICO PW ON PW.CD_DOCUMENTO_CLINICO = DA.CD_DOCUMENTO_CLINICO
  LEFT JOIN DBAMV.CID ON DA.CD_CID = CID.CD_CID;