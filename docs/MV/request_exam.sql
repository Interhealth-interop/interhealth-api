CREATE OR REPLACE VIEW "DBAMV"."REQUEST_EXAM"
(
    "request_exam_code",
    "request_exam_patient_code",
    "request_exam_encounter_code",
    "request_exam_provider_code",
    "request_exam_description",
    "request_exam_type",
    "request_exam_document",
    "request_exam_date",
    "request_exam_time",
    "request_exam_notes",
    "request_exam_category",
    "request_exam_status",
    "request_exam_provider_code",
    "request_exam_location_code",
    "request_exam_updated_date",
    "request_exam_reason",
    "request_exam_esponsible"
) AS
SELECT 
         COALESCE(el.CD_PRO_FAT, ex.EXA_RX_CD_PRO_FAT) AS "request_exam_code",
         P.CD_PACIENTE                                 AS "request_exam_patient_code",
         PM.CD_ATENDIMENTO                             AS "request_exam_encounter_code",
         PM.CD_PRESTADOR                               AS "request_exam_provider_code",
         TP.DS_TIP_PRESC                               AS "request_exam_description",
         NULL                                          AS "request_exam_type",
         IM.CD_PRE_MED                                 AS "request_exam_document",
         to_char(PM.DT_PRE_MED, 'YYYY-MM-DD')          AS "request_exam_date",
         to_char(PM.HR_PRE_MED, 'HH24:MI:SS')          AS "request_exam_time",
         IM.DS_JUSTIFICATIVA                           AS "request_exam_notes",
         TE.DS_TIP_ESQ                                 AS "request_exam_category",              
         CASE
              WHEN hc.DH_MEDICACAO IS NULL THEN 'não checado'
              ELSE 'realizado'
           END  AS "request_exam_status",
           NULL AS "request_exam_provider_code",
           NULL AS "request_exam_location_code",
           NULL AS "request_exam_updated_date",
           NULL AS "request_exam_reason",
           NULL AS "request_exam_esponsible"
       FROM       DBAMV.PRE_MED PM   
       INNER JOIN DBAMV.ITPRE_MED IM    ON IM.CD_PRE_MED = PM.CD_PRE_MED
       LEFT  JOIN dbamv.hritpre_cons hc         ON HC.CD_ITPRE_MED = IM.CD_ITPRE_MED
       INNER JOIN DBAMV.TIP_ESQ TE      ON IM.CD_TIP_ESQ = TE.CD_TIP_ESQ
       INNER JOIN DBAMV.TIP_PRESC TP    ON IM.CD_TIP_PRESC = TP.CD_TIP_PRESC
       LEFT  JOIN DBAMV.EXA_LAB EL      ON EL.CD_EXA_LAB = TP.CD_EXA_LAB
       LEFT  JOIN DBAMV.EXA_RX  EX      ON EX.CD_EXA_RX = TP.CD_EXA_RX
       INNER JOIN DBAMV.ATENDIME A      ON A.CD_ATENDIMENTO = PM.CD_ATENDIMENTO
       INNER JOIN DBAMV.PACIENTE P      ON P.CD_PACIENTE = A.CD_PACIENTE
       INNER JOIN DBAMV.PW_DOCUMENTO_CLINICO PW ON PW.CD_DOCUMENTO_CLINICO = PM.CD_DOCUMENTO_CLINICO    
       WHERE PW.TP_STATUS = 'ASSINADO'
        AND (TE.SN_EXA_RX = 'S' or SN_EXA_LAB = 'S');          
                                   