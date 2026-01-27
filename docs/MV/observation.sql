CREATE OR REPLACE VIEW "DBAMV"."OBSERVATION"
(
    "observation_code",
    "observation_encounter_code",
    "observation_patient_code",
    "observation_description",
    "observation_notes",
    "observation_provider_code",
    "observation_date",
    "observation_time",
    "observation_status",
    "observation_type",
    "observation_updated_date",
    "observation_reason",
    "observation_responsible"
) AS
SELECT 
    P.CD_PRE_MED AS "observation_code",
    P.CD_ATENDIMENTO AS "observation_encounter_code",
    A.CD_PACIENTE AS "observation_patient_code",
    PO.NM_OBJETO AS "observation_description",
    P.DS_EVOLUCAO AS "observation_notes",
    P.CD_PRESTADOR AS "observation_provider_code",
    TO_CHAR(P.HR_PRE_MED, 'YYYY-MM-DD') AS "observation_date",
    TO_CHAR(P.HR_PRE_MED, 'HH24:MI:SS') AS "observation_time",
    PW.TP_STATUS AS "observation_status",
    PO.NM_OBJETO AS "observation_type",
    NULL AS "observation_updated_date",
    NULL AS "observation_reason",
    NULL AS "observation_responsible"
FROM 
    DBAMV.PRE_MED P
    INNER JOIN PW_DOCUMENTO_CLINICO PW ON PW.CD_DOCUMENTO_CLINICO = P.CD_DOCUMENTO_CLINICO
    INNER JOIN DBAMV.ATENDIME A ON A.CD_ATENDIMENTO = P.CD_ATENDIMENTO
    INNER JOIN DBAMV.PAGU_OBJETO PO ON PO.CD_OBJETO = P.CD_OBJETO
WHERE 
    P.DS_EVOLUCAO IS NOT NULL
--  and po.cd_objeto in ( 'é necessário informar aqui os códigos dos objetos correspondentes do cliente que queira trazer conforme gorvenança.' )   -- com a linha comentada tras todos os documentos/observation
      
                 
           
